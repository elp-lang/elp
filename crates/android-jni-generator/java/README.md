* Install maven somehow
* Install Java (probably through SDKMan)
* `cd ./metadata-extractor/`
* Build with `mvn clean package`
* run against an android jar file like this `java -jar target/metadata-extractor.jar $ANDROID_HOME/platforms/android-36/android.jar output-jar-metadata.json`
