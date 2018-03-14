import java.io.File

import jobs.{CreateDictionary, WordCountTexts}
import org.apache.hadoop.conf.Configuration
import org.apache.log4j.{Level, Logger}
import org.apache.spark.{SparkConf, SparkContext}

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
          case Some(limit) => FileSampler.sampleDirectory(config.textsInputDirectory.getPath, limit).mkString(",")
          case None => config.textsInputDirectory.getPath
        }

        // Create a count of all words specified by input options
        val dictionaryWordCount = CreateDictionary.dictionaryWordCount(inputFilesDescriptor, stopwords, sc)
        val dictionary = CreateDictionary.dictionary(dictionaryWordCount)

        val filesToProcess = sc.wholeTextFiles(inputFilesDescriptor)
        val textWordCounts = WordCountTexts.countWordsInTexts(filesToProcess, dictionary, sc)

        val timestamp: Long = System.currentTimeMillis / 1000
        val stampedOutputDir = config.outputDirectory.toPath.resolve(s"run-${timestamp}").toFile
        stampedOutputDir.mkdirs

        // Write output to files
        dictionaryWordCount.saveAsTextFile(stampedOutputDir.toPath.resolve("dictionaryWordCount").toString)
        textWordCounts.saveAsTextFile(stampedOutputDir.toPath.resolve("textWordCounts").toString)
      }
    }
  }

  case class Config(stopWordFile: File = new File("."),
                    textsInputDirectory: File = new File("."),
                    outputDirectory: File = new File("."),
                    randomSampling: Option[Int] = None)

}
