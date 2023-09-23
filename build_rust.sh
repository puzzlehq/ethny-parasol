PACKAGE_NAME="sunscreen_ballot"
FRAMEWORK_NAME="Ballot"
TARGET_PLATFORM="aarch64-apple-ios"

# Project paths
IOS_PROJ=/Users/darvishkamalia/Desktop/sunscreen-ios/sunscreen_test/sunscreen_test/

rm -f target/universal.a
rm -rf build/ios/
rm -rf "${FRAMEWORK_NAME}".xcframework
rm -rf "${IOS_PROJ}/${FRAMEWORK_NAME}.xcframework"

cargo build --package $PACKAGE_NAME --target $TARGET_PLATFORM
cargo run --package $PACKAGE_NAME --bin uniffi-bindgen generate --library target/${TARGET_PLATFORM}/debug/lib${PACKAGE_NAME}.a --language swift --out-dir ./build/ios

mv "build/ios/${PACKAGE_NAME}FFI.modulemap" "build/ios/module.modulemap"
xcodebuild -create-xcframework \
    -library "target/${TARGET_PLATFORM}/debug/lib${PACKAGE_NAME}.a" -headers "build/ios/" \
    -output "${IOS_PROJ}/${FRAMEWORK_NAME}".xcframework
  rm -f "$IOS_PROJ/${PACKAGE_NAME}.swift"
  cp "build/ios/${PACKAGE_NAME}.swift" "$IOS_PROJ/"