pub mod cotahist;
pub mod cvm;
pub mod live;
pub mod sinacor;

#[cfg(test)]
mod tests {
    use super::cotahist::CotahistParser;
    use super::sinacor::SinacorParser;
    use b3_core::tax::OperationSide;

    #[test]
    fn test_sinacor_parser() {
        let note = "1-BOVESPA C VISTA PETR4 PETROBRAS 100 36,50 3.650,00 C";
        let trades = SinacorParser::parse_trade_lines(note, "Clear Corretora");
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        assert_eq!(t.symbol, "PETR4");
        assert_eq!(t.side, OperationSide::Buy);
        assert_eq!(t.quantity, 100.0);
        assert_eq!(t.unit_price, 36.50);
        assert!(t.liquidation_fee > 0.0);
    }

    #[test]
    fn test_cotahist_parser() {
        // Sample 245-byte COTAHIST record
        // Construct exact 245-byte slice
        let mut bytes = vec![b' '; 245];
        bytes[0..2].copy_from_slice(b"01");
        bytes[2..10].copy_from_slice(b"20261025");
        bytes[10..12].copy_from_slice(b"02");
        bytes[12..24].copy_from_slice(b"PETR4       ");
        bytes[56..69].copy_from_slice(b"0000000003500"); // PREABE 35.00
        bytes[69..82].copy_from_slice(b"0000000003650"); // PREMAX 36.50
        bytes[82..95].copy_from_slice(b"0000000003480"); // PREMIN 34.80
        bytes[95..108].copy_from_slice(b"0000000003580"); // PREMED 35.80
        bytes[108..121].copy_from_slice(b"0000000003620"); // PREULT 36.20
        bytes[152..170].copy_from_slice(b"000000000000001000"); // QUATOT
        bytes[170..188].copy_from_slice(b"000000000036200000"); // VOLTOT
        let line = String::from_utf8(bytes).unwrap();

        let parsed = CotahistParser::parse_line(&line);
        assert!(parsed.is_some());
        let (sym, candle) = parsed.unwrap();
        assert_eq!(sym, "PETR4");
        assert_eq!(candle.open, 35.0);
        assert_eq!(candle.high, 36.5);
        assert_eq!(candle.low, 34.8);
        assert_eq!(candle.close, 36.2);
    }
}

