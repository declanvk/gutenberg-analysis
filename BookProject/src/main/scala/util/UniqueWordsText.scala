package util

import org.apache.spark.SparkContext
import org.apache.spark.rdd.RDD

object UniqueWordsText {
    def countUniqueWords(textFile: String, dictionary: RDD[(String, Long)], sc: SparkContext): Int = {
      val text = sc.textFile(textFile)
      val words = text.flatMap(_.split("\\b+"))
        .map(filePair => filePair.trim.toLowerCase -> 1)  //remove spacings, set to lowercase. Change so word is the key
        .leftOuterJoin(dictionary)  // join dictionary words and words in text
        .filter(_._2._2.isDefined) // Remove words not in the dictionary
        .map {
        case (_, (book, Some(index))) =>  ((book, index), 1)
      }
        .groupByKey // group words together
        .map { // map each entry to key=bookID value=(wordIndex (in dict), timesWordOccurred (in book))
        case ((bookID, wordIndex), wordCounts) => bookID -> (wordIndex -> wordCounts.sum)
      }
        .groupByKey  //combine data by each book identifier
        .collect()

      words.length
    }
}
