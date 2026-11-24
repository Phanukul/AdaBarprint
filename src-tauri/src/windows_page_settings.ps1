$targetPaper = $null
$wantPaper = $(if ($cfg.paperSize) { [string]$cfg.paperSize } else { 'A4' })
$pwMm = $(if ($cfg.paperWidthMm) { [double]$cfg.paperWidthMm } else { 210.0 })
$phMm = $(if ($cfg.paperHeightMm) { [double]$cfg.paperHeightMm } else { 297.0 })
$landscape = $false
# Match physical stock dimensions. Drivers expose landscape A4/A5 as portrait stock plus orientation.
if ($wantPaper -notin @('ROLL', 'CUSTOM', 'SINGLE')) {
  $sizes = @($doc.PrinterSettings.PaperSizes)
  $targetPaper = $sizes | Where-Object {
    [Math]::Abs($_.Width * 0.254 - $pwMm) -lt 0.6 -and [Math]::Abs($_.Height * 0.254 - $phMm) -lt 0.6
  } | Select-Object -First 1
  if (!$targetPaper -and $pwMm -gt $phMm) {
    $targetPaper = $sizes | Where-Object {
      [Math]::Abs($_.Width * 0.254 - $phMm) -lt 0.6 -and [Math]::Abs($_.Height * 0.254 - $pwMm) -lt 0.6
    } | Select-Object -First 1
    $landscape = [bool]$targetPaper
  }
}
if (!$targetPaper) {
  $targetPaper = New-Object System.Drawing.Printing.PaperSize($wantPaper, [int][Math]::Round($pwMm / 0.254), [int][Math]::Round($phMm / 0.254))
}
$doc.PrinterSettings.DefaultPageSettings.PaperSize = $targetPaper
$doc.DefaultPageSettings.PaperSize = $targetPaper
$doc.PrinterSettings.DefaultPageSettings.Landscape = $landscape
$doc.DefaultPageSettings.Landscape = $landscape

$matchedTray = $null
if ($cfg.trayName -and $cfg.trayName.Trim() -ne '') {
  $wantTray = $cfg.trayName.Trim()
  $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -ieq $wantTray } | Select-Object -First 1
  if (-not $matchedTray) {
    $matchedTray = $doc.PrinterSettings.PaperSources | Where-Object { $_.SourceName.Trim() -like ($wantTray + '*') } | Select-Object -First 1
  }
  if ($matchedTray) {
    $doc.PrinterSettings.DefaultPageSettings.PaperSource = $matchedTray
    $doc.DefaultPageSettings.PaperSource = $matchedTray
  }
}
