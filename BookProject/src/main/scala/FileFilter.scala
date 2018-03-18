package jobs

import org.apache.log4j.{Level, Logger}
import org.apache.spark.{SparkConf, SparkContext}

object FileFilter {

  def main(args: Array[String]): Unit = {
    Logger.getLogger("org").setLevel(Level.OFF)
    Logger.getLogger("akka").setLevel(Level.OFF)

    val sparkConf = new SparkConf()
      .setAppName("gutenberg-analysis")
      .setMaster("local[4]")
    val sc = new SparkContext(sparkConf)

    // open the list of texts with there counts of unique words
    val uniqueLines = sc.textFile("../data/spark/uniqueWordCounts.txt")
    val uniques = uniqueLines.map(line => (line.split(" ")(0), line.split(" ")(1).toInt))
      .filter(x => x._2 < 8000)
      .filter(x => x._2 > 1500)
      .map(x => x._1.split("/")(3))

    // save the filtered list of texts
    uniques.coalesce(1).saveAsTextFile("../data/spark/filteredTexts.txt")
  }

}
