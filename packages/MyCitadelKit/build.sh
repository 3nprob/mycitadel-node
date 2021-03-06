xcodebuild archive -scheme MyCitadelKit -destination "generic/platform=iOS" -archivePath ../../artifacts/MyCitadelKit-iOS SKIP_INSTALL=NO BUILD_LIBRARY_FOR_DISTRIBUTION=YES &&
  xcodebuild archive -scheme MyCitadelKit -destination "generic/platform=iOS Simulator" -archivePath ../../artifacts/MyCitadelKit-iOS-Sim SKIP_INSTALL=NO BUILD_LIBRARY_FOR_DISTRIBUTION=YES VALID_ARCHS=x86_64 &&
  xcodebuild archive -scheme CitadelKit -destination "generic/platform=macOS" -archivePath ../../artifacts/MyCitadelKit-macOS SKIP_INSTALL=NO BUILD_LIBRARY_FOR_DISTRIBUTION=YES VALID_ARCHS=x86_64 &&
  cd ../../artifacts &&
  rm -rf ./MyCitadelKit.xcframework &&
  xcodebuild -create-xcframework -framework MyCitadelKit-iOS.xcarchive/Products/Library/Frameworks/MyCitadelKit.framework \
             -framework MyCitadelKit-iOS-Sim.xcarchive/Products/Library/Frameworks/MyCitadelKit.framework \
             -framework MyCitadelKit-macOS.xcarchive/Products/Library/Frameworks/MyCitadelKit.framework \
             -output ./MyCitadelKit.xcframework
