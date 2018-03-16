package main

import java.io.File
import java.nio.file.Paths

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

      opt[String]('t', "texts")
        .required()
        .valueName("<texts-directory>")
        .action((x, c) => c.copy(textsInputDirectory = x))
        .text("directory containing all texts to process")

      opt[String]('l', "listing")
        .required()
        .valueName("<texts-directory-listing>")
        .action((x, c) => c.copy(textsListing = x))
        .text("file containing a listing of all texts")

      opt[String]('o', "output")
        .required()
        .valueName("<output-directory>")
        .action((x, c) => c.copy(outputDirectory = x))
        .text("directory to save all results in")

      opt[String]('s', "stopwords")
        .required()
        .valueName("<stopwords-file>")
        .action((x, c) => c.copy(stopWordFile = x))
        .text("file containing stopwords")

      opt[DataMode.Value]('m', "mode")
        .required()
        .valueName("<data-mode>")
        .action((x, c) => c.copy(dataMode = x))
        .text("location and retrieval mode of data")

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
        val sc = new SparkContext(sparkConf)

        val hadoopConfig: Configuration = sc.hadoopConfiguration
        hadoopConfig.set("fs.hdfs.impl", classOf[org.apache.hadoop.hdfs.DistributedFileSystem].getName)
        hadoopConfig.set("fs.file.impl", classOf[org.apache.hadoop.fs.LocalFileSystem].getName)
        hadoopConfig.set("fs.s3a.impl", classOf[org.apache.hadoop.fs.s3a.S3AFileSystem].getName)

        if (config.dataMode == DataMode.S3) {
          val accessKeyID = System.getenv("AWS_ACCESS_KEY_ID")
          val secretAccessKey = System.getenv("AWS_SECRET_ACCESS_KEY")

          hadoopConfig.set("fs.s3a.access.key", accessKeyID)
          hadoopConfig.set("fs.s3a.secret.key", secretAccessKey)
        }

        // Read stopwords from file and create a broadcast variable
        val stopwords = CreateDictionary.stopwords(config.stopWordFile, sc, config.dataMode)

        // If random sample config is set, sample specific number of files
        // from inputDirectory and use those to create dictionary and
        // perform word count.
        // Else, use every file in input directory
        val inputFilesDescriptor: String = config.randomSampling match {
          case Some(limit) => {
            val listingFile = sc.textFile(config.textsListing)

            listingFile
              .sample(withReplacement = false, limit/listingFile.count().toDouble)
              .map(x => config.textsInputDirectory + x)
              .collect()
              .mkString(",")
          }
          case None => config.textsInputDirectory
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
        val stampedOutputDir = Paths.get(config.outputDirectory).resolve(s"run-${timestamp}").toFile
        stampedOutputDir.mkdirs

        dictionaryWordCount.saveAsTextFile(stampedOutputDir.toPath.resolve("dictionaryWordCount").toString)
        documentVectors.groupByKey.saveAsTextFile(stampedOutputDir.toPath.resolve("documentVectors").toString)
        similarityMatrix.coalesce(1).saveAsTextFile(stampedOutputDir.toPath.resolve("similarityMatrix").toString)
      }
    }
  }

  case class Config(stopWordFile: String = "",
                    textsInputDirectory: String = "",
                    textsListing: String = "",
                    outputDirectory: String = "",
                    dataMode: DataMode.Value = DataMode.Local,
                    randomSampling: Option[Int] = None)

  object DataMode extends Enumeration {
    val Local, S3 = Value
  }

  implicit val dataModeRead: scopt.Read[DataMode.Value] = scopt.Read.reads(DataMode withName _)
}
