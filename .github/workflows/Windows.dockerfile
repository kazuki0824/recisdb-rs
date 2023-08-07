ARG IMAGE=mcr.microsoft.com/dotnet/framework/runtime:4.8.1
FROM ${IMAGE}
SHELL ["powershell", "-command"]

RUN Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointsManager]::ServerCertificationCallback = {$true}; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
RUN choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System' -y
RUN choco install rustup.install llvm mingw -y
RUN choco install visualstudio2022buildtools -y
RUN choco install rustup.install -y

RUN rustup default stable-gnu