import java.io.File

import jobs.{CreateDictionary, WordCountTexts}
import org.apache.log4j.{Level, Logger}
import org.apache.spark.{SparkConf, SparkContext}

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

      opt[File]('s', "stop")
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
        val sparkConf = new SparkConf().setAppName("App").setMaster("local[4]")
        val sc = new SparkContext(sparkConf)

        val createDictionaryJob = new CreateDictionary(config.textsInputDirectory, config.stopWordFile, sc)
        val countWordsJob = new WordCountTexts(config.textsInputDirectory, createDictionaryJob, sc)

        countWordsJob.countWordsInTexts(countWordsJob.allFiles).collect().foreach(println)
      }
    }
  }

  case class Config(stopWordFile: File = new File("."),
                    textsInputDirectory: File = new File("."),
                    outputDirectory: File = new File("."),
                    randomSampling: Option[Int] = None)

}
