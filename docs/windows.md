# Windows

The only dependency of argc is bash. Developers under windows usually have [git](https://gitforwindows.org/) installed, and git has built-in bash. So you can safely use argc under windows.

## Make `.sh` file executable

If you want to run a `.sh` script file directly like a `.cmd` or `.exe` file, execute the following code in PowerShell.

```ps1
# Add .sh to PATHEXT
[Environment]::SetEnvironmentVariable("PATHEXT", [Environment]::GetEnvironmentVariable("PATHEXT", "Machine") + ";.SH", "Machine")

# Associate the .sh file extension with Git Bash
New-Item -LiteralPath Registry::HKEY_CLASSES_ROOT\.sh -Force
New-ItemProperty -LiteralPath Registry::HKEY_CLASSES_ROOT\.sh -Name "(Default)" -Value "sh_auto_file" -PropertyType String -Force
New-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Classes\sh_auto_file\shell\open\command' `
  -Name '(default)' -Value '"C:\Program Files\Git\bin\bash.exe" "%1" %*' -PropertyType String -Force
```

![image](https://github.com/sigoden/argc/assets/4012553/16af2b13-8c20-4954-bf58-ccdf1bbe23ef)
