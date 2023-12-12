param(
    [string]$sqliteFilePath1,
    [string]$sqliteFilePath2
)

$ErrorActionPreference = "Stop"

if (-not $sqliteFilePath1) {
    Exit 1
}

if (-not $sqliteFilePath2) {
  Exit 1
}

if (-not (Test-Path $sqliteFilePath1 -PathType Leaf)) {
    Write-Host "The specified file does not exist: $sqliteFilePath1"
    Exit 1
}

if (-not (Test-Path $sqliteFilePath2 -PathType Leaf)) {
  Write-Host "The specified file does not exist: $sqliteFilePath2"
  Exit 1
}

$filePath = C:\Users\SHIFTUP\bin\diff2d.exe $sqliteFilePath1 $sqliteFilePath2 2> C:\Users\SHIFTUP\bin\diff2d.log

$IS_SUCCESS = $LASTEXITCODE

# Check the exit code of the program
if (! $IS_SUCCESS -eq 0) {
  Write-Host "The program exited with code $IS_SUCCESS"
  Exit $IS_SUCCESS
}

Write-Host $filePath

# Create a new Excel application object
$excel = New-Object -ComObject Excel.Application

# Make Excel visible (optional, you can comment this line if you don't want Excel to be visible)
$excel.Visible = $true

# Open the Excel file
$workbook = $excel.Workbooks.Open($filePath)

# Cleanup - Release the Excel COM objects
[System.Runtime.Interopservices.Marshal]::ReleaseComObject($workbook) | Out-Null
[System.Runtime.Interopservices.Marshal]::ReleaseComObject($excel) | Out-Null

# Display a message indicating success
Write-Host "Excel file opened successfully: $filePath"
