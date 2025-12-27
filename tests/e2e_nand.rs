use nander_rs::application::use_cases::{
    EraseFlashUseCase, EraseParams, ReadFlashUseCase, ReadParams, WriteFlashUseCase, WriteParams,
};
use nander_rs::domain::{
    BadBlockStrategy, Capacity, ChipCapabilities, ChipLayout, ChipSpec, FlashType, JedecId, OobMode,
};
use nander_rs::infrastructure::flash_protocol::nand::SpiNand;
use nander_rs::infrastructure::programmer::simulator::SimulatedProgrammer;

#[test]
fn test_e2e_nand_lifecycle() {
    // 1. Setup Simulator
    // 128MB Flash, 2KB Page, 128KB Block
    let page_size = 2048;
    let block_size = 128 * 1024;
    let capacity_bytes = 128 * 1024 * 1024;

    let programmer = SimulatedProgrammer::new(capacity_bytes, page_size, block_size);

    // 2. Setup Chip Spec
    let spec = ChipSpec {
        name: "Simulated NAND".to_string(),
        manufacturer: "Simulated".to_string(),
        jedec_id: JedecId::new([0xEF, 0xAA, 0x21]), // ID matched by simulator
        flash_type: FlashType::Nand,
        capacity: Capacity::bytes(capacity_bytes as u32),
        layout: ChipLayout {
            page_size,
            block_size,
            oob_size: Some(64),
            is_dataflash: false,
        },
        capabilities: ChipCapabilities::default(),
        otp: None,
    };

    // 3. Create Protocol Layer
    let mut flash = SpiNand::new(programmer, spec);

    // 4. Create Use Cases
    // Note: In real app we might create new use cases per operation or reuse.
    // Here we need to borrow flash mutably multiple times.
    // Use cases take specific `flash`. Usually they take strictly ownership or mutable ref?
    // Checking `ReadFlashUseCase<F: FlashOperation>`: it holds `flash: F`.
    // If I pass `&mut SpiNand`, it should work because `impl FlashOperation for &mut T`?
    // Let's check `src/domain/flash_operation.rs`. It imports `FlashOperation`.
    // `impl FlashOperation for Box<dyn FlashOperation>` exists.
    // Does `impl FlashOperation for &mut T` exist? I don't recall adding it.
    // If not, I'll have to consume/return flash or clone programmer (not cloneable easily).
    // Or I can just call methods on `flash` directly for this test, mimicking what use cases do,
    // OR create a wrapper.
    // Actually, `ReadFlashUseCase::new(flash)` takes ownership of `flash`.
    // I need to use the use cases sequentially if I want to pass ownership back and forth?
    // Or implement `FlashOperation` for `&mut SpiNand`? That would be best practice.
    // Let's blindly try to use `&mut flash` and if it fails compilation, I will add the impl in `flash_operation.rs`.

    // TEST: Erase Block 0
    // Manually erasing first to ensure clean state (simulator inits to FF but good to test)
    println!("Step 1: Erasing Block 0");
    let mut erase_uc = EraseFlashUseCase::new(&mut flash);
    let erase_params = EraseParams {
        address: 0,
        length: block_size,
        bad_block_strategy: BadBlockStrategy::Fail,
        bbt: None,
    };
    erase_uc
        .execute(erase_params, |_| {})
        .expect("Erase failed");

    // TEST: Write Data to Page 0
    println!("Step 2: Writing Data to Page 0");
    let test_data = vec![0xAB; 512]; // Partial page write
    let mut write_uc = WriteFlashUseCase::new(&mut flash);
    let write_params = WriteParams {
        address: 0,
        data: &test_data,
        use_ecc: true,
        verify: true, // Internal verify
        ignore_ecc_errors: false,
        oob_mode: OobMode::None,
        bad_block_strategy: BadBlockStrategy::Fail,
        bbt: None,
        retry_count: 0,
    };
    write_uc
        .execute(write_params, |_| {})
        .expect("Write failed");

    // TEST: Read Back
    println!("Step 3: Reading Back");
    let mut read_uc = ReadFlashUseCase::new(&mut flash);
    let read_params = ReadParams {
        address: 0,
        length: 512,
        use_ecc: true,
        ignore_ecc_errors: false,
        oob_mode: OobMode::None,
        bad_block_strategy: BadBlockStrategy::Fail,
        bbt: None,
        retry_count: 0,
    };
    let read_data = read_uc.execute(read_params, |_| {}).expect("Read failed");

    assert_eq!(read_data, test_data, "Read data mismatch");

    // TEST: Read specific byte from raw simulated memory (Backdoor check)
    // We can't easily access the programmer inside `flash` inside `use_case`...
    // But since `flash` is `SpiNand` enclosing `Box<SimulatedProgrammer>`,
    // and we only have `&mut flash`, getting the inner programmer is hard without casting.
    // But we verified via read_uc, which uses SPI commands, so that's good enough E2E.
}
