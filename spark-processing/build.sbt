lazy val root = (project in file("."))
  .settings(
    name := "gutenberg-processing",
    version := "0.1",
    scalaVersion := "2.11.8",
    mainClass in Compile := Some("App")
  )

resolvers ++= Seq(
  "apache-snapshots" at "http://repository.apache.org/snapshots/"
)

libraryDependencies ++= Seq(
  "org.apache.hadoop" % "hadoop-aws" % "2.7.3",
  "org.apache.spark" %% "spark-core" % "2.2.0" % "provided",
  "com.github.scopt" %% "scopt" % "3.7.0"
)

assemblyMergeStrategy in assembly := {
  case PathList("META-INF", xs @ _*) => MergeStrategy.discard
  case x => MergeStrategy.first
}