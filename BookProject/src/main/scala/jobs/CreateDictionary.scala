package jobs

import java.io.File

import org.apache.spark.SparkContext
import org.apache.spark.broadcast.Broadcast
import org.apache.spark.rdd.RDD

import scala.io.Source

object CreateDictionary {

  def stopwords(stopwordFile: File, sc: SparkContext): Broadcast[Set[String]] =
    sc.broadcast(Source.fromFile(stopwordFile).getLines.map(_.trim).toSet)

  def dictionaryWordCount(inputFileDescriptor: String, stopwords: Broadcast[Set[String]], sc: SparkContext): RDD[(String, Long)] = {
    sc.textFile(inputFileDescriptor)
      .flatMap(_.split("\\b+")) //split all words by nonword characters
      .map(_.trim.toLowerCase) //set all words to lowercase
      .filter(_.matches("[a-zA-Z]{3,}")) // filter out non words of length less than 3
      .filter(!stopwords.value.contains(_)) // remove all words in removable list
      .map(x => (x, 1L)).reduceByKey(_ + _) // count occurences of key
      .filter(_._2 >= 3) //only keep words that occur at least three times
      .persist()
  }

  def dictionary(dictionaryWordCount: RDD[(String, Long)]): RDD[(String, Long)] =
    dictionaryWordCount.keys.zipWithIndex().persist()
}
