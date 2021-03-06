package jobs

import org.apache.spark.rdd.RDD

object CalculateSimilarity {
    def calculateDocumentMagnitude(documentVectors: RDD[(Int, (Long, Int))]): RDD[(Int, Float)] = {
        documentVectors
            .aggregateByKey(0)(
                (accum, x) => accum + (x._2 * x._2),
                (a, b) => a + b
            )
            .mapValues(x => Math.sqrt(x.toDouble).toFloat)
    }

    def calculateDotProduct(documentVectors: RDD[(Int, (Long, Int))]): RDD[((Int, Int), Long)] = {
        val rearrangedVectors = documentVectors.map {
            case (documentID, (wordIndex, wordCount)) => (wordIndex, (documentID, wordCount.toLong))
        }

        rearrangedVectors
            .join(rearrangedVectors) // RDD[(WordIdx, ((DocumentID_A, WordCount_A), (DocumentID_B, WordCount_B)))]
            .filter {
                case (_, (document_A, document_B)) => document_A._1 < document_B._1
            }
            .map {
                case (wordIndex, ((documentID_A, wordCount_A), (documentID_B, wordCount_B))) => {
                    (documentID_A, documentID_B) -> (wordCount_A * wordCount_B)
                }
            } // RDD[((DocumentID_A, DocumentID_B), PartialInnerProduct)]
            .reduceByKey((a, b) => a + b)
    }

    def calculateSimilarityMatrix(documentVectors: RDD[(Int, (Long, Int))]): RDD[((Int, Int), Float)] = {
        val magnitudes = CalculateSimilarity.calculateDocumentMagnitude(documentVectors)
        val dotProducts = CalculateSimilarity.calculateDotProduct(documentVectors)

        val cosineSimilarityDenominator = magnitudes
            .cartesian(magnitudes) // RDD[((DocumentID_A, DocumentMagnitude_A), (DocumentID_B, DocumentMagnitude_B))]
            .filter {
                case (document_A, document_B) => document_A._1 < document_B._1
            }
            .map {
                case ((documentID_A, documentMagnitude_A), (documentID_B, documentMagnitude_B)) => {
                    (documentID_A, documentID_B) -> (documentMagnitude_A * documentMagnitude_B)
                }
            }

        cosineSimilarityDenominator.join(dotProducts).mapValues {
            case (pairMagnitudes, dotProduct) => dotProduct / pairMagnitudes
        }
        .map {
            case (documentPair, similarity) if documentPair._1 != documentPair._2 => (documentPair, similarity)
            case (documentPair, similarity) => (documentPair, 1.0f)
        }
    }

    def findKNearest(similarityMatrix: RDD[((Int, Int), Float)], labelFile: RDD[(Int, Iterable[String])]): RDD[(Int, List[(Iterable[String], Float)])] = {
       similarityMatrix
          .flatMap { case ((docID_A, docID_B), simFact) => List((docID_A, (docID_B, simFact)), (docID_B, (docID_A, simFact)))}
          .map { case (docID_A, (docID_B, simFact)) =>  (docID_B, (docID_A, simFact))}
          .join(labelFile)
          .map { case (_, ((docID_A, simFact), (subjects))) => (docID_A, (subjects, simFact))}
          .groupByKey()
          .map { case(docID, list) => (docID, list.toList.sortWith((x,y) => x._2 > y._2).take(10)) }
    }

    def findSubjectMatch(kNearest: RDD[(Int, List[(Iterable[String], Float)])]): RDD[(Int, List[(String, Float)])] = {
        kNearest
          .map {
              case (docID, list) => (docID, list.flatMap(x => {
                  val fact = x._2
                  x._1.map(name => (name, fact))
              }))
          }
          .map {
              case (docID, list) => (docID, list.groupBy(x => x._1)
                .map {
                    case (key, values) => (key, values.map(_._2).sum / 10)
                }
                .toList
                .sortWith((a, b) => a._2 > b._2)
              )
          }
    }
}