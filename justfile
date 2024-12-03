set shell := ['powershell.exe', '-c']

r9x_toolchain := 'rust9x'
r9x_target := 'i586-rust9x-windows-msvc'
r9x_editbin := 'C:\Program Files\Microsoft Visual Studio\2022\Preview\VC\Tools\MSVC\14.43.34604\bin\Hostx64\x64\editbin.exe'

# These settings shoud work for Windows 95+ and NT 3.51+.
subsystem := 'CONSOLE,4.0'
os_version := '3.1'

build *FLAGS: (do-build 'debug' FLAGS)
release *FLAGS: (do-build 'release' '--release' FLAGS)
do-build PROFILE *FLAGS='': (r9x 'build' '--target' r9x_target FLAGS) (editbin 'target\'+r9x_target+'\'+PROFILE+'\rust9x_sample.exe')
run *FLAGS: (r9x 'run' '--target' r9x_target FLAGS)

r9x $COMMAND *FLAGS:
    cargo +{{ r9x_toolchain }} {{ COMMAND }} {{ FLAGS }}

# PE executables specify the subsystem and subsystem version as well as the required OS version.
#
# `link.exe` has the same switches, but doesn't allow setting them to values that lie outside of the
# supported range of the toolset. `editbin.exe` is a bit more forgiving, only warning about
# "invalid" values (`LINK : warning LNK4241: invalid subsystem version number 4`), but still
# carrying out the change as requested. `/OSVERSION` is entirely undocumented, but still works,
# setting the minimum required OS version. Both values must be in a supported range for the target
# OS to accept and run the executable.
editbin EXECUTABLE:
    & "{{ r9x_editbin }}" {{ EXECUTABLE }} /SUBSYSTEM:{{ subsystem }} /OSVERSION:{{ os_version }} /RELEASE /STACK:1048576
