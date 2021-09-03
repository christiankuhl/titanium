// use super::AHCI_CONTROLLERS;

// macro_rules! create_ahci_interrupt_handler {
//     ($irq:expr) => (
//         paste::paste! {
//             fn [<ahci_interrupt_handler_ $irq>](_stack_frame: &crate::interrupts::InterruptStackFrame, rsp: u64) -> u64 {
//                 AHCI_CONTROLLERS.lock()[$irq].handle_interrupt();
//                 unsafe {
//                     crate::drivers::pic::PICS.lock().notify_end_of_interrupt(crate::interrupts::Interrupt::AHCI as u8);
//                 }
//                 rsp
//             }
//         }
//     );
// }

// create_ahci_interrupt_handler!(11);
// create_ahci_interrupt_handler!(12);
// create_ahci_interrupt_handler!(13);
// create_ahci_interrupt_handler!(14);
// create_ahci_interrupt_handler!(15);

// #[macro_export]
// macro_rules! ahci_interrupt_handler {
//     ($irq:expr) => {
//         handler!(ahci_interrupt_handler_$irq);
//     };
// }
