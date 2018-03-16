package jobs

import java.io.File
import java.nio.file.Paths

import org.apache.spark.rdd.RDD

object WordCountTexts {
  def countWordsInTexts(textFiles: RDD[(String, String)], dictionary: RDD[(String, Long)]): RDD[(Int, (Long, Int))] = {
    textFiles.map {
        case (filename, contents) => {
            val textFilename = Paths.get(filename).getFileName.toString
            textFilename.substring(0, textFilename.lastIndexOf('.')).toInt -> contents
        }
      }
      .flatMapValues(_.split("\\b+"))
      .map(filePair => filePair._2.trim.toLowerCase -> filePair._1)  //remove spacings, set to lowercase. Change so word is the key
      .leftOuterJoin(dictionary)  // join dictionary words and words in text
      .filter(_._2._2.isDefined) // Remove words not in the dictionary
      .map {
        case (_, (book, Some(index))) =>  ((book, index), 1)
      }
      .reduceByKey((a, b) => a + b)
      .map {
        case ((bookID, wordIndex), totalCount) => bookID -> (wordIndex, totalCount)
      }
  }

}
