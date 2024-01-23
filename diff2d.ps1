param(
  [string]$sqliteFilePath1,
  [string]$sqliteFilePath2
)

[System.Reflection.Assembly]::LoadWithPartialName("System.Windows.Forms")

$ErrorActionPreference = "Stop"

if (-not $sqliteFilePath1) {
  Exit 1
}

if (-not $sqliteFilePath2) {
  Exit 1
}

if (-not (Test-Path $sqliteFilePath1 -PathType Leaf)) {
  [System.Windows.Forms.MessageBox]::Show("The specified file does not exist: $sqliteFilePath1", "Error", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Error)
  Exit 1
}


if (-not (Test-Path $sqliteFilePath2 -PathType Leaf)) {
  [System.Windows.Forms.MessageBox]::Show("The specified file does not exist: $sqliteFilePath2", "Error", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Error)
  Exit 1
}

$job = Start-Job -ScriptBlock { .\diff2d.exe $args[0] $args[1] 2> .\diff2d.log } -ArgumentList $sqliteFilePath1, $sqliteFilePath2

# open a progress bar window
$progressBar = New-Object System.Windows.Forms.Form
$progressBar.Text = "Please wait..."
$progressBar.Size = New-Object System.Drawing.Size(400,200)
$progressBar.FormBorderStyle = 'Fixed3D'
$progressBar.MaximizeBox = $false
$progressBar.MinimizeBox = $false
$progressBar.StartPosition = "CenterScreen"
$textBox = New-Object System.Windows.Forms.TextBox
$textBox.AutoSize = $true
$textBox.Size = New-Object System.Drawing.Size(200, 200)
$textBox.Multiline = $true
$textBox.Text = "1mb 당 1초 정도 소요됩니다."
$progressBar.Controls.Add($textBox)

$progressBar.Show()
Wait-Job -Job $job
$progressBar.Close()

# if job failed, show error message
if ($job.State -eq "Failed") {
  $errorMessage = $job.JobStateInfo.Reason.Message
  [System.Windows.Forms.MessageBox]::Show($errorMessage, "Error", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Error)
  Exit 1
}

# put job's output into a variable
$jobOutput = Receive-Job -Job $job

# when diff are same, jobOutput is actually 2 lines and first line is "same"
$is_same = $jobOutput -split "'n" | Select-Object -First 1
$filePath = $jobOutput -split "'n" | Select-Object -Last 1

if ($is_same -eq "same") {
  $result = [System.Windows.Forms.MessageBox]::Show("두 파일은 동일합니다. 그래도 비교 엑셀 파일을 열까요?", "Info", [System.Windows.Forms.MessageBoxButtons]::YesNo, [System.Windows.Forms.MessageBoxIcon]::Information)
  if ($result -ne "Yes") {
    Exit 0
  }
}

# Create a new Excel application object
$excel = New-Object -ComObject Excel.Application

# Make Excel visible (optional, you can comment this line if you don't want Excel to be visible)
$excel.Visible = $true

# Open the Excel file
$workbook = $excel.Workbooks.Open($filePath)

# Cleanup - Release the Excel COM objects
[System.Runtime.Interopservices.Marshal]::ReleaseComObject($workbook) | Out-Null
[System.Runtime.Interopservices.Marshal]::ReleaseComObject($excel) | Out-Null