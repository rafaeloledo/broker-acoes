use crate::ticker::AssetClass;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct B3Asset {
    pub ticker: &'static str,
    pub name: &'static str,
    pub sector: &'static str,
    pub asset_class: AssetClass,
    pub base_price: f64,
}

pub static B3_ASSET_CATALOG: &[B3Asset] = &[
    // -------------------------------------------------------------
    // AÇÕES B3 (Blue Chips & Small Caps)
    // -------------------------------------------------------------
    B3Asset { ticker: "PETR4", name: "Petrobras PN", sector: "Petróleo e Gás", asset_class: AssetClass::Acao, base_price: 36.40 },
    B3Asset { ticker: "PETR3", name: "Petrobras ON", sector: "Petróleo e Gás", asset_class: AssetClass::Acao, base_price: 39.80 },
    B3Asset { ticker: "VALE3", name: "Vale S.A. ON", sector: "Mineração e Siderurgia", asset_class: AssetClass::Acao, base_price: 58.90 },
    B3Asset { ticker: "ITUB4", name: "Itaú Unibanco PN", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 33.60 },
    B3Asset { ticker: "ITUB3", name: "Itaú Unibanco ON", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 29.40 },
    B3Asset { ticker: "BBDC4", name: "Banco Bradesco PN", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 13.90 },
    B3Asset { ticker: "BBDC3", name: "Banco Bradesco ON", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 12.30 },
    B3Asset { ticker: "BBAS3", name: "Banco do Brasil ON", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 26.80 },
    B3Asset { ticker: "WEGE3", name: "WEG S.A. ON", sector: "Bens Industriais & Motores", asset_class: AssetClass::Acao, base_price: 52.30 },
    B3Asset { ticker: "ABEV3", name: "Ambev S.A. ON", sector: "Bebidas & Consumo Não Cíclico", asset_class: AssetClass::Acao, base_price: 12.10 },
    B3Asset { ticker: "RENT3", name: "Localiza Rent a Car ON", sector: "Aluguel de Carros & Frotas", asset_class: AssetClass::Acao, base_price: 43.50 },
    B3Asset { ticker: "BPAC11", name: "BTG Pactual Unit", sector: "Bancos de Investimento", asset_class: AssetClass::Acao, base_price: 34.80 },
    B3Asset { ticker: "ITSA4", name: "Itaúsa Holding PN", sector: "Holdings Financeiras", asset_class: AssetClass::Acao, base_price: 10.45 },
    B3Asset { ticker: "ITSA3", name: "Itaúsa Holding ON", sector: "Holdings Financeiras", asset_class: AssetClass::Acao, base_price: 10.20 },
    B3Asset { ticker: "BBSE3", name: "BB Seguridade ON", sector: "Seguros & Previdência", asset_class: AssetClass::Acao, base_price: 35.10 },
    B3Asset { ticker: "PRIO3", name: "PRIO S.A. ON", sector: "Petróleo & Gás Independente", asset_class: AssetClass::Acao, base_price: 42.90 },
    B3Asset { ticker: "SUZB3", name: "Suzano S.A. ON", sector: "Papel & Celulose", asset_class: AssetClass::Acao, base_price: 54.20 },
    B3Asset { ticker: "GGBR4", name: "Gerdau S.A. PN", sector: "Siderurgia & Aço Longo", asset_class: AssetClass::Acao, base_price: 18.70 },
    B3Asset { ticker: "GGBR3", name: "Gerdau S.A. ON", sector: "Siderurgia & Aço Longo", asset_class: AssetClass::Acao, base_price: 16.50 },
    B3Asset { ticker: "CSNA3", name: "CSN Companhia Siderúrgica ON", sector: "Siderurgia & Mineração", asset_class: AssetClass::Acao, base_price: 11.80 },
    B3Asset { ticker: "JBSS3", name: "JBS S.A. ON", sector: "Carnes & Alimentos", asset_class: AssetClass::Acao, base_price: 32.70 },
    B3Asset { ticker: "BRFS3", name: "BRF S.A. ON", sector: "Alimentos Sadia e Perdigão", asset_class: AssetClass::Acao, base_price: 24.10 },
    B3Asset { ticker: "CPLE6", name: "Copel PNB", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 9.80 },
    B3Asset { ticker: "CPLE3", name: "Copel ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 9.10 },
    B3Asset { ticker: "CMIG4", name: "Cemig PN", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 11.20 },
    B3Asset { ticker: "CMIG3", name: "Cemig ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 14.50 },
    B3Asset { ticker: "ELET3", name: "Eletrobras ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 38.60 },
    B3Asset { ticker: "ELET6", name: "Eletrobras PNB", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 42.10 },
    B3Asset { ticker: "EQTL3", name: "Equatorial Energia ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 32.40 },
    B3Asset { ticker: "SBSP3", name: "Sabesp S.A. ON", sector: "Saneamento Básico", asset_class: AssetClass::Acao, base_price: 89.50 },
    B3Asset { ticker: "EMBR3", name: "Embraer S.A. ON", sector: "Aeroespacial & Defesa", asset_class: AssetClass::Acao, base_price: 48.30 },
    B3Asset { ticker: "MGLU3", name: "Magazine Luiza ON", sector: "Varejo & E-commerce", asset_class: AssetClass::Acao, base_price: 9.40 },
    B3Asset { ticker: "LREN3", name: "Lojas Renner ON", sector: "Varejo de Moda", asset_class: AssetClass::Acao, base_price: 17.60 },
    B3Asset { ticker: "RADL3", name: "Raia Drogasil ON", sector: "Saúde & Farmácias", asset_class: AssetClass::Acao, base_price: 25.80 },
    B3Asset { ticker: "VBBR3", name: "Vibra Energia ON", sector: "Distribuição de Combustíveis", asset_class: AssetClass::Acao, base_price: 23.40 },
    B3Asset { ticker: "HYPE3", name: "Hypera Pharma ON", sector: "Medicamentos & Farmacêutica", asset_class: AssetClass::Acao, base_price: 26.20 },
    B3Asset { ticker: "CCRO3", name: "Grupo CCR ON", sector: "Concessões Rodoviárias", asset_class: AssetClass::Acao, base_price: 12.80 },
    B3Asset { ticker: "RAIL3", name: "Rumo S.A. ON", sector: "Logística Ferroviária", asset_class: AssetClass::Acao, base_price: 19.50 },
    B3Asset { ticker: "VIVT3", name: "Vivo Telefônica Brasil ON", sector: "Telecomunicações", asset_class: AssetClass::Acao, base_price: 52.80 },
    B3Asset { ticker: "TIMS3", name: "TIM Brasil ON", sector: "Telecomunicações", asset_class: AssetClass::Acao, base_price: 17.90 },
    B3Asset { ticker: "KLBN11", name: "Klabin S.A. Unit", sector: "Papel & Embalagens", asset_class: AssetClass::Acao, base_price: 21.30 },
    B3Asset { ticker: "KLBN4", name: "Klabin S.A. PN", sector: "Papel & Embalagens", asset_class: AssetClass::Acao, base_price: 4.25 },
    B3Asset { ticker: "SANB11", name: "Banco Santander Unit", sector: "Financeiro & Bancos", asset_class: AssetClass::Acao, base_price: 27.80 },
    B3Asset { ticker: "ALOS3", name: "Allos Shopping Centers ON", sector: "Shopping Centers", asset_class: AssetClass::Acao, base_price: 22.40 },
    B3Asset { ticker: "MULT3", name: "Multiplan ON", sector: "Shopping Centers", asset_class: AssetClass::Acao, base_price: 24.90 },
    B3Asset { ticker: "CYRE3", name: "Cyrela Brazil Realty ON", sector: "Construção Civil & Incorporação", asset_class: AssetClass::Acao, base_price: 21.70 },
    B3Asset { ticker: "MRVE3", name: "MRV Engenharia ON", sector: "Construção Civil Econômica", asset_class: AssetClass::Acao, base_price: 7.30 },
    B3Asset { ticker: "EZTC3", name: "EZTec Empreendimentos ON", sector: "Construção Civil Residencial", asset_class: AssetClass::Acao, base_price: 14.20 },
    B3Asset { ticker: "USIM5", name: "Usiminas PNA", sector: "Siderurgia & Aços Planos", asset_class: AssetClass::Acao, base_price: 6.40 },
    B3Asset { ticker: "GOAU4", name: "Metalúrgica Gerdau PN", sector: "Siderurgia", asset_class: AssetClass::Acao, base_price: 10.90 },
    B3Asset { ticker: "CSAN3", name: "Cosan S.A. ON", sector: "Holding Agro & Energia", asset_class: AssetClass::Acao, base_price: 12.60 },
    B3Asset { ticker: "SLCE3", name: "SLC Agrícola ON", sector: "Agronegócio & Grãos", asset_class: AssetClass::Acao, base_price: 18.90 },
    B3Asset { ticker: "SMTO3", name: "São Martinho ON", sector: "Açúcar & Etanol", asset_class: AssetClass::Acao, base_price: 28.50 },
    B3Asset { ticker: "BEEF3", name: "Minerva Foods ON", sector: "Frigoríficos & Exportação", asset_class: AssetClass::Acao, base_price: 6.80 },
    B3Asset { ticker: "MRFG3", name: "Marfrig Global Foods ON", sector: "Carnes & Hambúrgueres", asset_class: AssetClass::Acao, base_price: 13.40 },
    B3Asset { ticker: "DXCO3", name: "Dexco Deca e Duratex ON", sector: "Painéis de Madeira & Louças", asset_class: AssetClass::Acao, base_price: 7.20 },
    B3Asset { ticker: "TOTS3", name: "Totvs S.A. ON", sector: "Software & Tecnologia ERP", asset_class: AssetClass::Acao, base_price: 28.90 },
    B3Asset { ticker: "POSI3", name: "Positivo Tecnologia ON", sector: "Hardware & Eletrônicos", asset_class: AssetClass::Acao, base_price: 6.50 },
    B3Asset { ticker: "LWSA3", name: "Locaweb Serviços ON", sector: "Tecnologia & E-commerce", asset_class: AssetClass::Acao, base_price: 4.60 },
    B3Asset { ticker: "CASH3", name: "Méliuz S.A. ON", sector: "Fintech & Cashback", asset_class: AssetClass::Acao, base_price: 6.10 },
    B3Asset { ticker: "ENGI11", name: "Energisa Unit", sector: "Distribuição Elétrica", asset_class: AssetClass::Acao, base_price: 45.60 },
    B3Asset { ticker: "TAEE11", name: "Taesa Unit", sector: "Transmissão Elétrica", asset_class: AssetClass::Acao, base_price: 34.50 },
    B3Asset { ticker: "TAEE4", name: "Taesa PN", sector: "Transmissão Elétrica", asset_class: AssetClass::Acao, base_price: 11.50 },
    B3Asset { ticker: "TAEE3", name: "Taesa ON", sector: "Transmissão Elétrica", asset_class: AssetClass::Acao, base_price: 11.45 },
    B3Asset { ticker: "EGIE3", name: "Engie Brasil ON", sector: "Geração de Energia Renovável", asset_class: AssetClass::Acao, base_price: 42.10 },
    B3Asset { ticker: "CPFE3", name: "CPFL Energia ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 35.80 },
    B3Asset { ticker: "TRPL4", name: "Isa Cteep Paulista PN", sector: "Transmissão de Energia", asset_class: AssetClass::Acao, base_price: 24.60 },
    B3Asset { ticker: "NEOE3", name: "Neoenergia ON", sector: "Energia Elétrica", asset_class: AssetClass::Acao, base_price: 18.90 },
    B3Asset { ticker: "CXSE3", name: "Caixa Seguridade ON", sector: "Seguros & Previdência", asset_class: AssetClass::Acao, base_price: 14.80 },
    B3Asset { ticker: "PSSA3", name: "Porto Seguro ON", sector: "Seguros de Automóvel e Saúde", asset_class: AssetClass::Acao, base_price: 35.40 },
    B3Asset { ticker: "B3SA3", name: "B3 Bolsa Balcão Brasil ON", sector: "Infraestrutura de Mercado", asset_class: AssetClass::Acao, base_price: 10.90 },
    B3Asset { ticker: "CIEL3", name: "Cielo S.A. ON", sector: "Adquirência & Pagamentos", asset_class: AssetClass::Acao, base_price: 5.60 },
    B3Asset { ticker: "COGN3", name: "Cogna Educação ON", sector: "Educação Superior & Básica", asset_class: AssetClass::Acao, base_price: 1.45 },
    B3Asset { ticker: "YDUQ3", name: "YDUQS Participações ON", sector: "Educação & Medicina", asset_class: AssetClass::Acao, base_price: 9.80 },
    B3Asset { ticker: "ASAI3", name: "Assaí Atacadista ON", sector: "Varejo & Atacarejo", asset_class: AssetClass::Acao, base_price: 7.60 },
    B3Asset { ticker: "CRFB3", name: "Carrefour Brasil ON", sector: "Varejo Alimentar", asset_class: AssetClass::Acao, base_price: 8.90 },
    B3Asset { ticker: "AZUL4", name: "Azul Linhas Aéreas PN", sector: "Aviação Comercial", asset_class: AssetClass::Acao, base_price: 5.30 },
    B3Asset { ticker: "GOLL4", name: "Gol Linhas Aéreas PN", sector: "Aviação Comercial", asset_class: AssetClass::Acao, base_price: 1.15 },
    B3Asset { ticker: "CMIN3", name: "CSN Mineração ON", sector: "Minério de Ferro", asset_class: AssetClass::Acao, base_price: 5.80 },
    B3Asset { ticker: "BRAP4", name: "Bradespar PN", sector: "Holding de Mineração", asset_class: AssetClass::Acao, base_price: 18.20 },
    B3Asset { ticker: "FLRY3", name: "Grupo Fleury ON", sector: "Diagnósticos & Laboratórios", asset_class: AssetClass::Acao, base_price: 15.30 },
    B3Asset { ticker: "HAPV3", name: "Hapvida NotreDame ON", sector: "Operadoras de Saúde", asset_class: AssetClass::Acao, base_price: 3.80 },
    B3Asset { ticker: "RDOR3", name: "Rede D'Or São Luiz ON", sector: "Hospitais & Oncologia", asset_class: AssetClass::Acao, base_price: 29.50 },
    B3Asset { ticker: "ODPV3", name: "Odontoprev ON", sector: "Planos Odontológicos", asset_class: AssetClass::Acao, base_price: 11.70 },
    B3Asset { ticker: "VAMO3", name: "Grupo Vamos ON", sector: "Locação de Caminhões", asset_class: AssetClass::Acao, base_price: 6.20 },
    B3Asset { ticker: "RANI3", name: "Irani Papel e Embalagem ON", sector: "Papel Reciclado & Embalagens", asset_class: AssetClass::Acao, base_price: 7.90 },
    B3Asset { ticker: "MDIA3", name: "M. Dias Branco ON", sector: "Massas, Biscoitos e Trigo", asset_class: AssetClass::Acao, base_price: 27.50 },
    B3Asset { ticker: "CURY3", name: "Cury Construtora ON", sector: "Construção Civil SP e RJ", asset_class: AssetClass::Acao, base_price: 21.80 },
    B3Asset { ticker: "DIRR3", name: "Direcional Engenharia ON", sector: "Construção Minha Casa", asset_class: AssetClass::Acao, base_price: 28.40 },
    B3Asset { ticker: "PLPL3", name: "Plano&Plano ON", sector: "Construção Civil Econômica", asset_class: AssetClass::Acao, base_price: 12.30 },
    B3Asset { ticker: "JHSF3", name: "JHSF Alta Renda ON", sector: "Hotéis, Shoppings e Aeroporto", asset_class: AssetClass::Acao, base_price: 4.10 },
    B3Asset { ticker: "LOGG3", name: "LOG CP Galpões ON", sector: "Desenvolvimento Logístico", asset_class: AssetClass::Acao, base_price: 20.90 },
    B3Asset { ticker: "SIMH3", name: "Simpar Holding ON", sector: "Transporte & Logística", asset_class: AssetClass::Acao, base_price: 5.70 },
    B3Asset { ticker: "STBP3", name: "Santos Brasil ON", sector: "Portos & Terminais de Contêiner", asset_class: AssetClass::Acao, base_price: 12.95 },
    B3Asset { ticker: "MOVI3", name: "Movida Aluguel de Carros ON", sector: "Locação de Veículos", asset_class: AssetClass::Acao, base_price: 6.40 },
    B3Asset { ticker: "ECOR3", name: "EcoRodovias ON", sector: "Concessões Rodoviárias", asset_class: AssetClass::Acao, base_price: 7.50 },
    B3Asset { ticker: "BRKM5", name: "Braskem PNA", sector: "Petroquímica & Termoplásticos", asset_class: AssetClass::Acao, base_price: 17.80 },
    B3Asset { ticker: "UNIP6", name: "Unipar Carbocloro PNB", sector: "Química & Cloro Soda", asset_class: AssetClass::Acao, base_price: 56.40 },
    B3Asset { ticker: "RECV3", name: "PetroRecôncavo ON", sector: "Petróleo Onshore", asset_class: AssetClass::Acao, base_price: 17.50 },
    B3Asset { ticker: "BRAV3", name: "Brava Energia ON", sector: "Petróleo & Exploração", asset_class: AssetClass::Acao, base_price: 16.90 },
    B3Asset { ticker: "VIVA3", name: "Vivara Joias ON", sector: "Joalheria & Luxo", asset_class: AssetClass::Acao, base_price: 24.80 },
    B3Asset { ticker: "GUAR3", name: "Riachuelo Guararapes ON", sector: "Varejo de Moda & Midway", asset_class: AssetClass::Acao, base_price: 7.40 },
    B3Asset { ticker: "CEAB3", name: "C&A Modas ON", sector: "Varejo de Moda", asset_class: AssetClass::Acao, base_price: 11.80 },
    B3Asset { ticker: "BHIA3", name: "Casas Bahia ON", sector: "Eletrodomésticos & Móveis", asset_class: AssetClass::Acao, base_price: 4.20 },
    B3Asset { ticker: "PCAR3", name: "Pão de Açúcar GPA ON", sector: "Supermercados", asset_class: AssetClass::Acao, base_price: 2.90 },
    B3Asset { ticker: "NTCO3", name: "Natura &Co ON", sector: "Cosméticos & Beleza", asset_class: AssetClass::Acao, base_price: 14.20 },
    B3Asset { ticker: "AGRO3", name: "BrasilAgro ON", sector: "Fazendas & Grãos", asset_class: AssetClass::Acao, base_price: 23.90 },
    B3Asset { ticker: "TTEN3", name: "3tentos Agro ON", sector: "Insumos & Grãos", asset_class: AssetClass::Acao, base_price: 11.40 },
    B3Asset { ticker: "CAML3", name: "Camil Alimentos ON", sector: "Arroz, Feijão e Açúcar", asset_class: AssetClass::Acao, base_price: 8.60 },
    B3Asset { ticker: "KEPL3", name: "Kepler Weber ON", sector: "Armazenagem & Silos Agrícolas", asset_class: AssetClass::Acao, base_price: 9.30 },
    B3Asset { ticker: "TUPY3", name: "Tupy S.A. ON", sector: "Metalurgia & Blocos de Motor", asset_class: AssetClass::Acao, base_price: 22.10 },
    B3Asset { ticker: "ROMI3", name: "Indústrias Romi ON", sector: "Máquinas e Equipamentos", asset_class: AssetClass::Acao, base_price: 10.80 },
    B3Asset { ticker: "LEVE3", name: "Mahle Metal Leve ON", sector: "Autopeças & Pistões", asset_class: AssetClass::Acao, base_price: 33.40 },
    B3Asset { ticker: "FRAS3", name: "Frasle Mobility ON", sector: "Pastilhas & Lonas de Freio", asset_class: AssetClass::Acao, base_price: 18.50 },
    B3Asset { ticker: "RAPT4", name: "Randoncorp PN", sector: "Implementos & Autopeças", asset_class: AssetClass::Acao, base_price: 10.70 },
    B3Asset { ticker: "POMO4", name: "Marcopolo PN", sector: "Carrocerias de Ônibus", asset_class: AssetClass::Acao, base_price: 8.10 },
    B3Asset { ticker: "INTB3", name: "Intelbras ON", sector: "Câmeras, Redes & Energia Solar", asset_class: AssetClass::Acao, base_price: 19.80 },
    B3Asset { ticker: "SAPR11", name: "Sanepar Unit", sector: "Saneamento do Paraná", asset_class: AssetClass::Acao, base_price: 28.90 },
    B3Asset { ticker: "SAPR4", name: "Sanepar PN", sector: "Saneamento do Paraná", asset_class: AssetClass::Acao, base_price: 5.65 },
    B3Asset { ticker: "CSMG3", name: "Copasa Minas Gerais ON", sector: "Saneamento de MG", asset_class: AssetClass::Acao, base_price: 21.40 },
    B3Asset { ticker: "ALUP11", name: "Alupar Investimento Unit", sector: "Transmissão Elétrica", asset_class: AssetClass::Acao, base_price: 30.50 },
    B3Asset { ticker: "AMBP3", name: "Ambipar Gestão Ambiental ON", sector: "Emergências Ambientais", asset_class: AssetClass::Acao, base_price: 88.00 },
    B3Asset { ticker: "SOJA3", name: "Boa Safra Sementes ON", sector: "Sementes de Soja", asset_class: AssetClass::Acao, base_price: 12.80 },
    B3Asset { ticker: "VULC3", name: "Vulcabras Mizuno & Olympikus ON", sector: "Calçados Esportivos", asset_class: AssetClass::Acao, base_price: 17.50 },
    B3Asset { ticker: "GRND3", name: "Grendene Melissa ON", sector: "Calçados Sintéticos", asset_class: AssetClass::Acao, base_price: 6.20 },
    B3Asset { ticker: "CVCB3", name: "CVC Brasil Viagens ON", sector: "Turismo & Passagens", asset_class: AssetClass::Acao, base_price: 1.85 },

    // -------------------------------------------------------------
    // FIIS B3 (Fundos de Investimento Imobiliário & Fiagros)
    // -------------------------------------------------------------
    B3Asset { ticker: "HGLG11", name: "CSHG Logística FII", sector: "Galpões Logísticos", asset_class: AssetClass::Fii, base_price: 159.20 },
    B3Asset { ticker: "KNCR11", name: "Kinea Rendimentos Imobiliários FII", sector: "Títulos e Valores Mobiliários (CRI)", asset_class: AssetClass::Fii, base_price: 104.50 },
    B3Asset { ticker: "MXRF11", name: "Maxi Renda FII", sector: "Híbrido e CRIs", asset_class: AssetClass::Fii, base_price: 10.15 },
    B3Asset { ticker: "XPML11", name: "XP Malls FII", sector: "Shopping Centers", asset_class: AssetClass::Fii, base_price: 108.40 },
    B3Asset { ticker: "BTLG11", name: "BTG Pactual Logística FII", sector: "Galpões Logísticos", asset_class: AssetClass::Fii, base_price: 99.80 },
    B3Asset { ticker: "VISC11", name: "Vinci Shopping Centers FII", sector: "Shopping Centers", asset_class: AssetClass::Fii, base_price: 114.20 },
    B3Asset { ticker: "KNRI11", name: "Kinea Renda Imobiliária FII", sector: "Misto (Lajes & Logística)", asset_class: AssetClass::Fii, base_price: 147.80 },
    B3Asset { ticker: "HGRU11", name: "CSHG Renda Urbana FII", sector: "Renda Urbana & Varejo", asset_class: AssetClass::Fii, base_price: 123.50 },
    B3Asset { ticker: "HGBS11", name: "Hedge Brasil Shopping FII", sector: "Shopping Centers", asset_class: AssetClass::Fii, base_price: 215.00 },
    B3Asset { ticker: "BRCO11", name: "Bresco Logística FII", sector: "Logística Triple A", asset_class: AssetClass::Fii, base_price: 117.60 },
    B3Asset { ticker: "XPLG11", name: "XP Log FII", sector: "Galpões Logísticos", asset_class: AssetClass::Fii, base_price: 101.40 },
    B3Asset { ticker: "VILG11", name: "Vinci Logística FII", sector: "Galpões Logísticos", asset_class: AssetClass::Fii, base_price: 94.80 },
    B3Asset { ticker: "PVBI11", name: "VBI Prime Properties FII", sector: "Lajes Corporativas", asset_class: AssetClass::Fii, base_price: 98.30 },
    B3Asset { ticker: "JSRE11", name: "JS Real Estate Lajes FII", sector: "Lajes Corporativas", asset_class: AssetClass::Fii, base_price: 68.20 },
    B3Asset { ticker: "RBRR11", name: "RBR Rendimento High Grade FII", sector: "CRI High Grade", asset_class: AssetClass::Fii, base_price: 92.40 },
    B3Asset { ticker: "CPTS11", name: "Capitânia Securities II FII", sector: "Papel e CRIs", asset_class: AssetClass::Fii, base_price: 8.25 },
    B3Asset { ticker: "KNHY11", name: "Kinea High Yield FII", sector: "CRI High Yield", asset_class: AssetClass::Fii, base_price: 99.80 },
    B3Asset { ticker: "IRDM11", name: "Iridium Recebíveis Imobiliários FII", sector: "Papel e CRIs", asset_class: AssetClass::Fii, base_price: 72.10 },
    B3Asset { ticker: "VGIR11", name: "Valora RE III FII", sector: "CRI Indexado ao CDI", asset_class: AssetClass::Fii, base_price: 9.60 },
    B3Asset { ticker: "RECR11", name: "REC Recebíveis Imobiliários FII", sector: "Papel e CRIs", asset_class: AssetClass::Fii, base_price: 81.30 },
    B3Asset { ticker: "TGAR11", name: "TG Ativo Real FII", sector: "Desenvolvimento Urbano", asset_class: AssetClass::Fii, base_price: 119.50 },
    B3Asset { ticker: "MALL11", name: "Malls Brasil Plural FII", sector: "Shopping Centers", asset_class: AssetClass::Fii, base_price: 112.40 },
    B3Asset { ticker: "LVBI11", name: "VBI Logístico FII", sector: "Galpões Industriais", asset_class: AssetClass::Fii, base_price: 114.70 },
    B3Asset { ticker: "RBRP11", name: "RBR Properties Lajes FII", sector: "Lajes Corporativas", asset_class: AssetClass::Fii, base_price: 51.60 },
    B3Asset { ticker: "BCFF11", name: "BTG Pactual Fundo de Fundos FII", sector: "Fund of Funds", asset_class: AssetClass::Fii, base_price: 8.95 },
    B3Asset { ticker: "VRTA11", name: "Fator Veritá FII", sector: "Papel & CRIs", asset_class: AssetClass::Fii, base_price: 86.40 },
    B3Asset { ticker: "VINO11", name: "Vinci Offices Lajes FII", sector: "Lajes Corporativas", asset_class: AssetClass::Fii, base_price: 6.80 },
    B3Asset { ticker: "GGRC11", name: "GGR Covepi Renda FII", sector: "Galpões e Indústria", asset_class: AssetClass::Fii, base_price: 11.20 },
    B3Asset { ticker: "TRXF11", name: "TRX Real Estate FII", sector: "Renda Urbana & Varejo", asset_class: AssetClass::Fii, base_price: 104.90 },
    B3Asset { ticker: "ALZR11", name: "Alianza Trust Renda Imobiliária FII", sector: "Atípicos e Sale-Leaseback", asset_class: AssetClass::Fii, base_price: 108.60 },
    B3Asset { ticker: "GALG11", name: "Guardian Logística FII", sector: "Galpões Logísticos", asset_class: AssetClass::Fii, base_price: 9.15 },
    B3Asset { ticker: "SNCI11", name: "Suno Recebíveis Imobiliários FII", sector: "CRI High Grade", asset_class: AssetClass::Fii, base_price: 94.30 },
    B3Asset { ticker: "SNFF11", name: "Suno Fundo de Fundos FII", sector: "Fund of Funds", asset_class: AssetClass::Fii, base_price: 86.80 },
    B3Asset { ticker: "MCCI11", name: "Mauá Capital Recebíveis FII", sector: "CRI High Grade", asset_class: AssetClass::Fii, base_price: 89.40 },
    B3Asset { ticker: "HGCR11", name: "CSHG Recebíveis Imobiliários FII", sector: "CRI High Grade", asset_class: AssetClass::Fii, base_price: 101.80 },
    B3Asset { ticker: "KNSC11", name: "Kinea Securities FII", sector: "CRI Indexado ao CDI", asset_class: AssetClass::Fii, base_price: 8.85 },
    B3Asset { ticker: "RURA11", name: "Itaú Asset Rural Fiagro", sector: "Fiagro Crédito Agro", asset_class: AssetClass::Fii, base_price: 9.90 },
    B3Asset { ticker: "VGIA11", name: "Valora CRA Fiagro", sector: "Fiagro CRA Agro", asset_class: AssetClass::Fii, base_price: 8.75 },
    B3Asset { ticker: "KNCA11", name: "Kinea Crédito Agro Fiagro", sector: "Fiagro CRA Agro", asset_class: AssetClass::Fii, base_price: 101.20 },
    B3Asset { ticker: "XPCA11", name: "XP Crédito Agrícola Fiagro", sector: "Fiagro CRA Agro", asset_class: AssetClass::Fii, base_price: 8.90 },
    B3Asset { ticker: "SNAG11", name: "Suno Agro Fiagro", sector: "Fiagro Crédito Agro", asset_class: AssetClass::Fii, base_price: 10.05 },
    B3Asset { ticker: "BTAL11", name: "BTG Agro Logística FII", sector: "Silos & Logística Agro", asset_class: AssetClass::Fii, base_price: 74.50 },

    // -------------------------------------------------------------
    // ETFS & BDRS
    // -------------------------------------------------------------
    B3Asset { ticker: "BOVA11", name: "iShares Ibovespa ETF", sector: "Índice B3 Ibovespa", asset_class: AssetClass::Etf, base_price: 125.40 },
    B3Asset { ticker: "IVVB11", name: "iShares S&P 500 ETF", sector: "Índice S&P 500 EUA", asset_class: AssetClass::Etf, base_price: 338.50 },
    B3Asset { ticker: "SMAL11", name: "iShares Small Cap ETF", sector: "Índice Small Caps B3", asset_class: AssetClass::Etf, base_price: 98.20 },
    B3Asset { ticker: "HASH11", name: "Hashdex Nasdaq Crypto Index ETF", sector: "Criptoativos & Bitcoin", asset_class: AssetClass::Etf, base_price: 64.80 },
    B3Asset { ticker: "GOLD11", name: "Trend ETF Ouro FIM", sector: "Ouro Físico Spot", asset_class: AssetClass::Etf, base_price: 15.40 },
    B3Asset { ticker: "NASD11", name: "Trend ETF Nasdaq 100", sector: "Índice Nasdaq 100 EUA", asset_class: AssetClass::Etf, base_price: 19.80 },
    B3Asset { ticker: "DIVO11", name: "It Now IDIV Dividendos ETF", sector: "Índice de Dividendos B3", asset_class: AssetClass::Etf, base_price: 104.20 },
    B3Asset { ticker: "BRAX11", name: "iShares IBrX 100 ETF", sector: "Índice IBrX 100", asset_class: AssetClass::Etf, base_price: 118.00 },
    B3Asset { ticker: "AAPL34", name: "Apple Inc. BDR", sector: "Hardware & iPhones", asset_class: AssetClass::Bdr, base_price: 64.50 },
    B3Asset { ticker: "MSFT34", name: "Microsoft Corp. BDR", sector: "Software, Nuvem & IA", asset_class: AssetClass::Bdr, base_price: 98.40 },
    B3Asset { ticker: "NVDC34", name: "NVIDIA Corp. BDR", sector: "GPUs & Chips de Inteligência Artificial", asset_class: AssetClass::Bdr, base_price: 18.20 },
    B3Asset { ticker: "AMZO34", name: "Amazon.com Inc. BDR", sector: "Cloud AWS & E-Commerce", asset_class: AssetClass::Bdr, base_price: 49.30 },
    B3Asset { ticker: "GOOGL34", name: "Alphabet Inc. Google BDR", sector: "Mecanismo de Busca & YouTube", asset_class: AssetClass::Bdr, base_price: 76.50 },
    B3Asset { ticker: "META34", name: "Meta Platforms Inc. BDR", sector: "Instagram, WhatsApp & Facebook", asset_class: AssetClass::Bdr, base_price: 122.00 },
    B3Asset { ticker: "TSLA34", name: "Tesla Inc. BDR", sector: "Carros Elétricos & Automação", asset_class: AssetClass::Bdr, base_price: 52.80 },
    B3Asset { ticker: "MELI34", name: "MercadoLibre Inc. BDR", sector: "E-Commerce & Mercado Pago", asset_class: AssetClass::Bdr, base_price: 112.50 },
    B3Asset { ticker: "NU34", name: "Nu Holdings Ltd. Nubank BDR", sector: "Fintech & Bancos Digitais", asset_class: AssetClass::Bdr, base_price: 14.80 },
];

pub fn get_all_b3_assets() -> &'static [B3Asset] {
    B3_ASSET_CATALOG
}

pub fn normalize(s: &str) -> String {
    s.to_uppercase()
        .chars()
        .map(|c| match c {
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            other => other,
        })
        .collect()
}

pub fn search_b3_assets(query: &str, filter_class: Option<AssetClass>) -> Vec<&'static B3Asset> {
    let q = normalize(query.trim());
    B3_ASSET_CATALOG
        .iter()
        .filter(|asset| {
            if let Some(fc) = filter_class {
                if asset.asset_class != fc {
                    return false;
                }
            }
            if q.is_empty() {
                return true;
            }
            let ticker_matches = normalize(asset.ticker).contains(&q);
            let name_matches = normalize(asset.name).contains(&q);
            let sector_matches = normalize(asset.sector).contains(&q);
            ticker_matches || name_matches || sector_matches
        })
        .collect()
}

pub fn find_b3_asset(ticker: &str) -> Option<&'static B3Asset> {
    let t = ticker.trim().to_uppercase();
    B3_ASSET_CATALOG.iter().find(|a| a.ticker.eq_ignore_ascii_case(&t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_b3_assets() {
        let results = search_b3_assets("PETR", None);
        assert!(results.len() >= 2);
        assert!(results.iter().any(|a| a.ticker == "PETR4"));

        let fii_results = search_b3_assets("LOGISTICA", Some(AssetClass::Fii));
        assert!(fii_results.iter().any(|a| a.ticker == "HGLG11" || a.ticker == "BTLG11"));

        let exact = find_b3_asset("VALE3");
        assert!(exact.is_some());
        assert_eq!(exact.unwrap().name, "Vale S.A. ON");
    }
}
