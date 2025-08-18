use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating virtual device...");
    let mut device = VirtualDevice::default()?;
    println!("Virtual device created successfully!");
    
    println!("Testing key press...");
    device.click(KEY_A)?;
    println!("Sent key A successfully!");
    
    println!("Testing mouse movement...");
    device.move_mouse(10, 10)?;
    println!("Mouse moved successfully!");
    
    println!("Testing mouse click...");
    device.click(BTN_LEFT)?;
    println!("Mouse clicked successfully!");
    
    println!("All tests passed!");
    Ok(())
}