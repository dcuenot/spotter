# This script takes care of packaging the build artifacts that will go in the
# release zipfile

set PATH=%PATH%;C:\Users\appveyor\.multirust\toolchains\i686-pc-windows-gnu\bin

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"

# TODO Update this to package the right artifacts
Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\invaders.exe" '.\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR
