package jobs

import org.apache.spark.rdd.RDD

object CalculateSimilarity {
    def calculateDocumentMagnitude(documentVectors: RDD[(String, (Long, Int))]): RDD[(String, Double)] = {
        documentVectors
            .aggregateByKey(0)(
                (accum, x) => accum + (x._2 * x._2),
                (a, b) => a + b
            )
            .mapValues(Math.sqrt(_))
            .persist
    }

    def calculateDotProduct(documentVectors: RDD[(String, (Long, Int))]): RDD[((String, String), Long)] = {
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
            .persist
    }

    def calculateSimilarityMatrix(documentVectors: RDD[(String, (Long, Int))]): RDD[((String, String), Double)] = {
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
        }.persist
    }
}