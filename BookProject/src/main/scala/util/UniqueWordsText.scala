package util

import java.io.File

import org.apache.spark.SparkContext
import org.apache.spark.broadcast.Broadcast

object UniqueWordsText {
    def countUniqueWords(inputFileDescriptor: File, stopwords: Broadcast[Set[String]], sc: SparkContext): Int = {
      val words = sc.textFile(inputFileDescriptor.toString)
        .flatMap(_.split("\\b+")) //split all words by nonword characters
        .map(_.trim.toLowerCase) //set all words to lowercase
        .filter(_.matches("[a-zA-Z]{3,}")) // filter out non words of length less than 3
        .filter(!stopwords.value.contains(_)) // remove all words in removable list
        .map(x => (x, 1L)).reduceByKey(_ + _) // count occurences of key
        .filter(_._2 >= 3) //only keep words that occur at least three times //combine data by each book identifier
        .collect()

      words.length
    }


}
