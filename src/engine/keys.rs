use xkeysym::Keysym;

pub fn translate_logical_key(key: Keysym) -> Option<u64> {
    Some(match key {
        Keysym::BackSpace => 0x00100000008,
        Keysym::Tab => 0x00100000009,
        Keysym::Return => 0x0010000000d,
        Keysym::Escape => 0x0010000001b,
        Keysym::Delete => 0x0010000007f,
        Keysym::Mode_switch => 0x00100000103, // AltGraph
        Keysym::Caps_Lock => 0x00100000104,
        // Fn doesn't have a direct equivalent in XKeysym
        // FnLock doesn't have a direct equivalent in XKeysym
        // Hyper doesn't have a direct equivalent in XKeysym
        Keysym::Num_Lock => 0x0010000010a,
        Keysym::Scroll_Lock => 0x0010000010c,
        Keysym::Super_L | Keysym::Super_R => 0x0010000010e,
        // Symbol doesn't have a direct equivalent in XKeysym
        // SymbolLock doesn't have a direct equivalent in XKeysym

        // Arrow keys
        Keysym::Down => 0x00100000301,
        Keysym::Left => 0x00100000302,
        Keysym::Right => 0x00100000303,
        Keysym::Up => 0x00100000304,
        Keysym::End => 0x00100000305,
        Keysym::Home => 0x00100000306,
        Keysym::Page_Down => 0x00100000307,
        Keysym::Page_Up => 0x00100000308,

        // Function keys
        Keysym::F1 => 0x00100000801,
        Keysym::F2 => 0x00100000802,
        Keysym::F3 => 0x00100000803,
        Keysym::F4 => 0x00100000804,
        Keysym::F5 => 0x00100000805,
        Keysym::F6 => 0x00100000806,
        Keysym::F7 => 0x00100000807,
        Keysym::F8 => 0x00100000808,
        Keysym::F9 => 0x00100000809,
        Keysym::F10 => 0x0010000080a,
        Keysym::F11 => 0x0010000080b,
        Keysym::F12 => 0x0010000080c,
        Keysym::F13 => 0x0010000080d,
        Keysym::F14 => 0x0010000080e,
        Keysym::F15 => 0x0010000080f,
        Keysym::F16 => 0x00100000810,
        Keysym::F17 => 0x00100000811,
        Keysym::F18 => 0x00100000812,
        Keysym::F19 => 0x00100000813,
        Keysym::F20 => 0x00100000814,
        Keysym::F21 => 0x00100000815,
        Keysym::F22 => 0x00100000816,
        Keysym::F23 => 0x00100000817,
        Keysym::F24 => 0x00100000818,

        // Modifier keys
        Keysym::Control_L | Keysym::Control_R => 0x00200000100,
        Keysym::Shift_L | Keysym::Shift_R => 0x00200000102,
        Keysym::Alt_L | Keysym::Alt_R => 0x00200000104,
        Keysym::Meta_L | Keysym::Meta_R => 0x00200000106,

        // For Character keys, we can use the Keysym value directly
        key if key >= Keysym::space && key <= Keysym::asciitilde => key.key_char().unwrap() as u64,

        // Unidentified key
        _ => return None,
    })
}

pub fn translate_physical_key(keysym: Keysym) -> Option<u64> {
    Some(match keysym {
        Keysym::Escape => 0x00000009,
        Keysym::exclam => 0x0000000a,      // digit1 with shift
        Keysym::at => 0x0000000b,          // digit2 with shift
        Keysym::numbersign => 0x0000000c,  // digit3 with shift
        Keysym::dollar => 0x0000000d,      // digit4 with shift
        Keysym::percent => 0x0000000e,     // digit5 with shift
        Keysym::asciicircum => 0x0000000f, // digit6 with shift
        Keysym::ampersand => 0x00000010,   // digit7 with shift
        Keysym::asterisk => 0x00000011,    // digit8 with shift
        Keysym::parenleft => 0x00000012,   // digit9 with shift
        Keysym::parenright => 0x00000013,  // digit0 with shift
        Keysym::minus => 0x00000014,
        Keysym::equal => 0x00000015,
        Keysym::BackSpace => 0x00000016,
        Keysym::Tab => 0x00000017,
        Keysym::q | Keysym::Q => 0x00000018,
        Keysym::w | Keysym::W => 0x00000019,
        Keysym::e | Keysym::E => 0x0000001a,
        Keysym::r | Keysym::R => 0x0000001b,
        Keysym::t | Keysym::T => 0x0000001c,
        Keysym::y | Keysym::Y => 0x0000001d,
        Keysym::u | Keysym::U => 0x0000001e,
        Keysym::i | Keysym::I => 0x0000001f,
        Keysym::o | Keysym::O => 0x00000020,
        Keysym::p | Keysym::P => 0x00000021,
        Keysym::bracketleft => 0x00000022,
        Keysym::bracketright => 0x00000023,
        Keysym::Return => 0x00000024,
        Keysym::Control_L => 0x00000025,
        Keysym::a | Keysym::A => 0x00000026,
        Keysym::s | Keysym::S => 0x00000027,
        Keysym::d | Keysym::D => 0x00000028,
        Keysym::f | Keysym::F => 0x00000029,
        Keysym::g | Keysym::G => 0x0000002a,
        Keysym::h | Keysym::H => 0x0000002b,
        Keysym::j | Keysym::J => 0x0000002c,
        Keysym::k | Keysym::K => 0x0000002d,
        Keysym::l | Keysym::L => 0x0000002e,
        Keysym::semicolon => 0x0000002f,
        Keysym::apostrophe => 0x00000030,
        Keysym::grave => 0x00000031,
        Keysym::Shift_L => 0x00000032,
        Keysym::backslash => 0x00000033,
        Keysym::z | Keysym::Z => 0x00000034,
        Keysym::x | Keysym::X => 0x00000035,
        Keysym::c | Keysym::C => 0x00000036,
        Keysym::v | Keysym::V => 0x00000037,
        Keysym::b | Keysym::B => 0x00000038,
        Keysym::n | Keysym::N => 0x00000039,
        Keysym::m | Keysym::M => 0x0000003a,
        Keysym::comma => 0x0000003b,
        Keysym::period => 0x0000003c,
        Keysym::slash => 0x0000003d,
        Keysym::Shift_R => 0x0000003e,
        Keysym::KP_Multiply => 0x0000003f,
        Keysym::Alt_L => 0x00000040,
        Keysym::space => 0x00000041,
        Keysym::Caps_Lock => 0x00000042,
        Keysym::F1 => 0x00000043,
        Keysym::F2 => 0x00000044,
        Keysym::F3 => 0x00000045,
        Keysym::F4 => 0x00000046,
        Keysym::F5 => 0x00000047,
        Keysym::F6 => 0x00000048,
        Keysym::F7 => 0x00000049,
        Keysym::F8 => 0x0000004a,
        Keysym::F9 => 0x0000004b,
        Keysym::F10 => 0x0000004c,
        Keysym::Num_Lock => 0x0000004d,
        Keysym::Scroll_Lock => 0x0000004e,
        Keysym::KP_7 => 0x0000004f,
        Keysym::KP_8 => 0x00000050,
        Keysym::KP_9 => 0x00000051,
        Keysym::KP_Subtract => 0x00000052,
        Keysym::KP_4 => 0x00000053,
        Keysym::KP_5 => 0x00000054,
        Keysym::KP_6 => 0x00000055,
        Keysym::KP_Add => 0x00000056,
        Keysym::KP_1 => 0x00000057,
        Keysym::KP_2 => 0x00000058,
        Keysym::KP_3 => 0x00000059,
        Keysym::KP_0 => 0x0000005a,
        Keysym::KP_Decimal => 0x0000005b,
        // Keysym::lang5 => 0x0000005d,
        Keysym::yen => 0x0000005e,
        Keysym::F11 => 0x0000005f,
        Keysym::F12 => 0x00000060,
        // Keysym::intlRo => 0x00000061,
        // Keysym::lang3 => 0x00000062,
        // Keysym::lang4 => 0x00000063,
        // Keysym::convert => 0x00000064,
        // Keysym::kanaMode => 0x00000065,
        // Keysym::nonConvert => 0x00000066,
        Keysym::KP_Enter => 0x00000068,
        Keysym::Control_R => 0x00000069,
        Keysym::KP_Divide => 0x0000006a,
        Keysym::Print => 0x0000006b,
        Keysym::Alt_R => 0x0000006c,
        Keysym::Home => 0x0000006e,
        Keysym::Up => 0x0000006f,
        Keysym::Page_Up => 0x00000070,
        Keysym::Left => 0x00000071,
        Keysym::Right => 0x00000072,
        Keysym::End => 0x00000073,
        Keysym::Down => 0x00000074,
        Keysym::Page_Down => 0x00000075,
        Keysym::Insert => 0x00000076,
        Keysym::Delete => 0x00000077,
        Keysym::KP_Equal => 0x0000007d,
        // Keysym::numpadSignChange => 0x0000007e,
        Keysym::Pause => 0x0000007f,
        Keysym::KP_Separator => 0x00000081,
        // Keysym::lang1 => 0x00000082,
        // Keysym::lang2 => 0x00000083,
        // Keysym::intlYen => 0x00000084,
        Keysym::Super_L => 0x00000085,
        Keysym::Super_R => 0x00000086,
        Keysym::Menu => 0x00000087,
        Keysym::Redo => 0x00000089,
        Keysym::Undo => 0x0000008b,
        Keysym::Select => 0x0000008c,
        Keysym::Find => 0x00000090,
        Keysym::Help => 0x00000092,
        Keysym::F13 => 0x000000bf,
        Keysym::F14 => 0x000000c0,
        Keysym::F15 => 0x000000c1,
        Keysym::F16 => 0x000000c2,
        Keysym::F17 => 0x000000c3,
        Keysym::F18 => 0x000000c4,
        Keysym::F19 => 0x000000c5,
        Keysym::F20 => 0x000000c6,
        Keysym::F21 => 0x000000c7,
        Keysym::F22 => 0x000000c8,
        Keysym::F23 => 0x000000c9,
        Keysym::F24 => 0x000000ca,

        // Handle basic ASCII characters (0x20-0x7e)
        key if (key.key_char().unwrap() as u64) >= 0x20
            && (key.key_char().unwrap() as u64) <= 0x7e =>
        {
            // For simple ASCII characters, we can use the keysym value directly
            key.key_char().unwrap() as u64
        }

        _ => return None,
    })
}
