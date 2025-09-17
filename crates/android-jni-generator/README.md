1. First build the java module that pulls all the JAR data out into a JSON file.
	> See ./java/README.md for details
2. run `cargo run -p android-jni-generator ./crates/android-jni-generator/java/metadata-extractor/output-ui-metadata.json > crates/android-jni/src/android_jar.rs` to generate rust bindings
3. run `cargo build -p android-jni` to see how badly it got fucked up because this is a nightmare.
