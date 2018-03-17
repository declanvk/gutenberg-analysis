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
        }.persist

        rearrangedVectors
            .join(rearrangedVectors) // RDD[(WordIdx, ((DocumentID_A, WordCount_A), (DocumentID_B, WordCount_B)))]
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
            .map {
                case ((documentID_A, documentMagnitude_A), (documentID_B, documentMagnitude_B)) => {
                    (documentID_A, documentID_B) -> (documentMagnitude_A * documentMagnitude_B)
                }
            }

        dotProducts.join(cosineSimilarityDenominator).mapValues {
            case (dotProduct, pairMagnitudes) => dotProduct / pairMagnitudes
        }
        .map {
            case (documentPair, similarity) if documentPair._1 != documentPair._2 => (documentPair, similarity)
            case (documentPair, similarity) => (documentPair, 1.0f)
        }
    }
}