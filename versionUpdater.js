const splitPackagePart = (packageFile) => {
  const splitIdx = packageFile.indexOf("[[bin]]");

  return [packageFile.substr(0, splitIdx), packageFile.substr(splitIdx)];
};

const hasVersion = (packagePart) => packagePart.includes("version");

const detectVersion = (packageFile) => {
  // Take just the `[package]` part of the file so we
  // wouldn't accidentally work with versions of dependencies
  const packagePart = splitPackagePart(packageFile)[0];

  if (!hasVersion(packagePart)) return "0.0.0";

  const version = packagePart
    // Find the version string in the `[package]` part
    // (an array is returned, we need the first item from it)
    .match(/version = '[\d\.]+'/)[0]
    // Then replace everything that's not a digit or a period
    .replace(/[^\d\.]/g, "");

  return version;
};

module.exports.readVersion = (packageFile) => {
  return detectVersion(packageFile);
};

module.exports.writeVersion = (packageFile, newVersion) => {
  const packageFileSplit = splitPackagePart(packageFile);
  // Take just the `[package]` part of the file so we
  // wouldn't accidentally overwrite any other versions
  // stored in packageFile
  let newPackagePart = packageFileSplit[0];

  // If there isn't any `version` record in the file
  if (!hasVersion(newPackagePart)) {
    // There's a bunch of whitespace at the end of the `[package]` part
    const allChars = newPackagePart.match(/\S/gm);
    const lastCharIdx = newPackagePart.lastIndexOf(
      allChars[allChars.length - 1]
    );

    // Insert the version into the correct spot
    newPackagePart = [
      newPackagePart.substr(0, lastCharIdx + 1),
      `\nversion = '${newVersion}'`,
      newPackagePart.substr(lastCharIdx + 1),
    ].join("");
  } else {
    const oldVersion = detectVersion(packageFile);

    newPackagePart = newPackagePart.replace(oldVersion, newVersion);
  }

  return newPackagePart + packageFileSplit[1];
};
