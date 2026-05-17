function prompt {
    try {
        $lastCommand = Get-History -Count 1 -ErrorAction SilentlyContinue
        if ($lastCommand -and $lastCommand.CommandLine -notlike "*status*") {
            $eventData = @{
                type = "terminal_command"
                content = $lastCommand.CommandLine
                metadata = @{
                    exitCode = if ($?) { 0 } else { 1 }
                    startTime = $lastCommand.StartExecutionTime
                    endTime = $lastCommand.EndExecutionTime
                }
            }
            
            $body = $eventData | ConvertTo-Json -Compress
            $cfPort = if ($env:CF_PORT) { $env:CF_PORT } else { "49152" }
            Invoke-RestMethod -Uri "http://127.0.0.1:$cfPort/event" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 1 -ErrorAction SilentlyContinue
        }
    } catch {
        # Ignore errors to keep prompt clean
    }

    "PS $($executionContext.SessionState.Path.CurrentLocation)> "
}
