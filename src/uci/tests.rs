#[cfg(test)]
mod commands {
    use std::io::Cursor;

    use crate::uci::Command;
    use crate::UCI;

    #[test]
    fn uci() {
        let mut uci = UCI::new("ace", "Excse");

        let mut reader = Cursor::new("   uci ");
        let mut writer = Vec::<u8>::new();

        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(matches!(result, Ok(None)));

        let output = String::from_utf8(writer).unwrap();
        let output: Vec<String> = output.lines().map(String::from).collect();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0], "id name ace");
        assert_eq!(output[1], "id author Excse");
        assert_eq!(output[2], "uciok");
    }

    #[test]
    fn debug() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.debug, false);

        let mut writer = Vec::<u8>::new();

        let mut reader = Cursor::new("   debug ");
        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(result.is_err());

        let mut reader = Cursor::new("  debug   toggle   ");
        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(result.is_err());

        let mut reader = Cursor::new(" debug   on ");
        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(matches!(result, Ok(None)));
        assert_eq!(uci.debug, true);

        let mut reader = Cursor::new("    debug   off  ");
        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(matches!(result, Ok(None)));
        assert_eq!(uci.debug, false);
    }

    #[test]
    fn isready() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.debug, false);

        let mut reader = Cursor::new(" isready  ");
        let mut writer = Vec::<u8>::new();

        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(matches!(result, Ok(Some(Command::IsReady))));

        let result = uci.send_readyok(&mut writer);
        assert!(matches!(result, Ok(None)));

        let output = String::from_utf8(writer).unwrap();
        let output: Vec<String> = output.lines().map(String::from).collect();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], "readyok");
    }

    #[test]
    fn quit() {
        let mut uci = UCI::new("ace", "Excse");

        let mut reader = Cursor::new("   quit ");
        let mut writer = Vec::<u8>::new();

        let result = uci.receive_command(&mut reader, &mut writer);
        assert!(matches!(result, Ok(Some(Command::Quit))));
    }
}
