package jobs

import java.io.File
import java.nio.file.Paths

import org.apache.spark.SparkContext
import org.apache.spark.rdd.RDD

class WordCountTexts(allTextsDirectory: File, createDictJob: CreateDictionary, sc: SparkContext) {

  def allFiles: RDD[(String, String)] = sc.wholeTextFiles(allTextsDirectory.getPath)

//  Sample N files from allTextsDirectory and use those
//  def sampleFiles(num: Int): RDD[(String, String)] = {
//
//  }

  def countWordsInTexts(textFiles: RDD[(String, String)]): RDD[(String, Iterable[(Long, Int)])] = {
    textFiles.map {
        case (filename, contents) => {
            Paths.get(filename).getFileName.toString -> contents
        }
      }
      .flatMapValues(_.split("\\b+"))
      .map(filePair => filePair._2.trim.toLowerCase -> filePair._1)  //remove spacings, set to lowercase. Change so word is the key
      .leftOuterJoin(createDictJob.dictionary)  // join dictionary words and words in text
      .filter(_._2._2.isDefined) // Remove words not in the dictionary
      .map {
        case (word, (book, Some(index))) =>  ((book, index), 1)
      }
      .groupByKey // group words together
      .map { // map each entry to key=bookID value=(wordIndex (in dict), timesWordOccurred (in book))
        case ((bookID, wordIndex), wordCounts) => bookID -> (wordIndex -> wordCounts.sum)
      }
      .groupByKey  //combine data by each book identifier
      .persist
  }

}
