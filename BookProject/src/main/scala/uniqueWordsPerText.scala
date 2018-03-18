import java.io._

import jobs.{CreateDictionary, WordCountTexts}
import org.apache.hadoop.conf.Configuration
import org.apache.log4j.{Level, Logger}
import org.apache.spark.{SparkConf, SparkContext}
import util.UniqueWordsText

object uniqueWordsPerText {

  def main(args: Array[String]): Unit = {
    Logger.getLogger("org").setLevel(Level.OFF)
    Logger.getLogger("akka").setLevel(Level.OFF)

    val sparkConf = new SparkConf()
      .setAppName("gutenberg-analysis")
      .setMaster("local[4]")
    val sc = new SparkContext(sparkConf)

    // open stopwords file
    val stopwords = CreateDictionary.stopwords(new File("../data/stopwords.txt"), sc)

    // get a list of all texts
    var files = List[File]()
    val d = new File("../data/texts/")
    if (d.exists && d.isDirectory) {
      files = d.listFiles.filter(_.isFile).toList
    } else {
      List[File]()
    }

    // for each text write the name and number of unique words
    val file = "../data/spark/uniqueWordCounts.txt"
    val writer = new BufferedWriter(new FileWriter(file))

    files.foreach(x => writer.write(x.toString + " " + UniqueWordsText.countUniqueWords(x, stopwords, sc) + "\n"))

    writer.close()

  }

}
