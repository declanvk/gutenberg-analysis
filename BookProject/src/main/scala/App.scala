import java.io.File

import jobs.{CalculateSimilarity, CreateDictionary, WordCountTexts}
import org.apache.hadoop.conf.Configuration
import org.apache.log4j.{Level, Logger}
import org.apache.spark.{SparkConf, SparkContext}
import org.apache.spark.rdd.RDD
import util.FileSampler

object App {
  def main(args: Array[String]): Unit = {
    Logger.getLogger("org").setLevel(Level.OFF)
    Logger.getLogger("akka").setLevel(Level.OFF)

    val parser = new scopt.OptionParser[Config]("gutenberg-analysis") {
      override def showUsageOnError = true

      help("help")
      version("version")
      head("gutenberg-analysis", "0.1")

      opt[File]('t', "texts")
        .required()
        .valueName("<texts-directory>")
        .action((x, c) => c.copy(textsInputDirectory = x))
        .validate(x =>
          if (x.exists()) success
          else failure("input text directory must exist"))
        .text("directory containing all texts to process")

      opt[File]('o', "output")
        .required()
        .valueName("<output-directory>")
        .action((x, c) => c.copy(outputDirectory = x))
        .text("directory to save all results in")

      opt[File]('s', "stopwords")
        .required()
        .valueName("<stopwords-file>")
        .action((x, c) => c.copy(stopWordFile = x))
        .text("file containing stopwords")

      opt[Int]('r', "sample")
        .optional()
        .valueName("<number-random-samples>")
        .action((x, c) => c.copy(randomSampling = Some(x)))
        .text("optional random sampling to limit")
    }

    parser.parse(args, Config()) match {
      case None => sys.exit(1)
      case Some(config) => {
        val sparkConf = new SparkConf()
          .setAppName("gutenberg-analysis")
          .setMaster("local[4]")
        val sc = new SparkContext(sparkConf)

        val hadoopConfig: Configuration = sc.hadoopConfiguration
        hadoopConfig.set("fs.hdfs.impl", classOf[org.apache.hadoop.hdfs.DistributedFileSystem].getName)
        hadoopConfig.set("fs.file.impl", classOf[org.apache.hadoop.fs.LocalFileSystem].getName)

        // Read stopwords from file and create a broadcast variable
        val stopwords = CreateDictionary.stopwords(config.stopWordFile, sc)

        // If random sample config is set, sample specific number of files
        // from inputDirectory and use those to create dictionary and
        // perform word count.
        // Else, use every file in input directory
        val inputFilesDescriptor: String = config.randomSampling match {
          case Some(limit) => {
            val listingFile = sc.textFile(config.textsInputDirectory.getPath + "/texts_listing.txt")  //get listing file

            listingFile
              .sample(withReplacement = false, limit/listingFile.count().toDouble)
              .map(x => config.textsInputDirectory.getPath + "/texts/" + x)
              .collect().mkString(",")

          }
          case None => config.textsInputDirectory.getPath
        }


        //  dictionaryWordCount: RDD[(Word, WordCount)]
        val dictionaryWordCount: RDD[(String, Long)] = CreateDictionary.dictionaryWordCount(inputFilesDescriptor, stopwords, sc).persist

        //  dictionary: // RDD[(Word, WordIdx)]
        val dictionary: RDD[(String, Long)] = CreateDictionary.dictionary(dictionaryWordCount)

        //  filesToProcess: RDD[(Filename, FileContents)]
        val filesToProcess: RDD[(String, String)] = sc.wholeTextFiles(inputFilesDescriptor)

        //  documentVectors: RDD[(DocumentID, (WordIdx, WordCount))]
        val documentVectors: RDD[(Int, (Long, Int))] = WordCountTexts.countWordsInTexts(filesToProcess, dictionary).persist

        //  similarityMatrix: RDD[((DocumentID_A, DocumentID_B), SimilarityMeasure)]
        val similarityMatrix: RDD[((Int, Int), Double)] = CalculateSimilarity.calculateSimilarityMatrix(documentVectors)

        // Write output to files
        val timestamp: Long = System.currentTimeMillis / 1000
        val stampedOutputDir = config.outputDirectory.toPath.resolve(s"run-${timestamp}").toFile
        stampedOutputDir.mkdirs

        dictionaryWordCount.saveAsTextFile(stampedOutputDir.toPath.resolve("dictionaryWordCount").toString)
        documentVectors.groupByKey.saveAsTextFile(stampedOutputDir.toPath.resolve("documentVectors").toString)
        similarityMatrix.coalesce(1).saveAsTextFile(stampedOutputDir.toPath.resolve("similarityMatrix").toString)
      }
    }
  }

  case class Config(stopWordFile: File = new File("."),
                    textsInputDirectory: File = new File("."),
                    outputDirectory: File = new File("."),
                    randomSampling: Option[Int] = None)

}
