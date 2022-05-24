# Source: https://github.com/LABSN/sound-ci-helpers/blob/master/windows/setup_sound.ps1

# Someday if this breaks we can use:
# https://github.com/duncanthrax/scream/tree/master/Install
# But it signs using a SHA that is buggy with Windows 7 so don't use for now
# https://support.microsoft.com/en-us/help/2921916/the-untrusted-publisher-dialog-box-appears-when-you-install-a-driver-i
function DownloadVirtualSoundcard () {
    $webclient = New-Object System.Net.WebClient
    $url = "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack43.zip"
    $filepath = $pwd.Path + "\vbcable.zip"
    Write-Host "Downloading" $url
    $retry_attempts = 2
    for($i=0; $i -lt $retry_attempts; $i++){
        try {
            $webclient.DownloadFile($url, $filepath)
            break
        }
        Catch [Exception]{
            Start-Sleep 1
        }
    }
    if (Test-Path $filepath) {
        Write-Host "File saved at" $filepath
    } else {
        # Retry once to get the error message if any at the last try
        $webclient.DownloadFile($url, $filepath)
    }
}

function main () {
    Push-Location $PSScriptRoot
    DownloadVirtualSoundcard
    Expand-Archive -LiteralPath vbcable.zip -DestinationPath vbcable
    certutil -addstore "TrustedPublisher" vbcable.cer
    # PnPutil.exe -i -a vbcable/vbMmeCable64_win7.inf
    .\devcon.exe install vbcable\vbMmeCable64_win7.inf VBAudioVACWDM
    Pop-location
}

main
