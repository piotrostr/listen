export interface WorldMiniApp {
  app_id: string;
  world_app_description: string;
  overview: string;
  name: string;
  category_ranking: number;
  logo_img_url: string;
}

export const miniapps: Record<string, WorldMiniApp[]> = {
  Tokens: [
    {
      app_id: "grants",
      world_app_description: "Worldcoin airdrops",
      overview: "Claim your Worldcoin ",
      name: "Worldcoin",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_d2905e660b94ad24d6fc97816182ab35/510bb4cc-a607-4dbd-9d07-3701b63ac0c7.png",
    },
    {
      app_id: "app_f1e44837a5e3c2af4da8925b46027645",
      world_app_description: "Claim your ORO daily on World Chain",
      overview:
        "ORO is a token every human can claim daily. Claim your ORO daily on World Chain, and invite friends to earn more ORO. ",
      name: "ORO",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_f1e44837a5e3c2af4da8925b46027645/6ddb6949-c475-4f14-b141-5bf6837d0abb.jpg",
    },
    {
      app_id: "app_3876b5f39809a50bb5ebe97c997bbcf1",
      world_app_description: "Claim your ORB tokens.",
      overview:
        "ORB is the first token on World Chain, distributed equally to every human. Every human can claim 1,000 ORB once in their lifetime. Claims are open forever and never expire. Universal distribution powered by World ID. One token, for every human. \n\nORB is an independent project and is not affiliated with Tools for Humanity or the Worldcoin Foundation. ",
      name: "ORB",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_3876b5f39809a50bb5ebe97c997bbcf1/fd6bc912-0015-4e0c-81e4-15fe700edb8e.png",
    },
    {
      app_id: "app_ee968e983074cb090e6f12cd75b63bb3",
      world_app_description: "Crack your egg daily to earn reward",
      overview: "Claim your daily egg rewards before they are gone ",
      name: "Eggs Vault",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_ee968e983074cb090e6f12cd75b63bb3/ed41e251-f754-43f7-8f4b-eb1ed62da0a8.png",
    },
    {
      app_id: "app_66c83ab8c851fb1e54b1b1b62c6ce39d",
      world_app_description: "Basic income and global democracy",
      overview:
        "An open, democratic community dedicated to solving humanity\u2019s most pressing global challenges, with an own digital currency to support its initiatives. ",
      name: "Republic",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_66c83ab8c851fb1e54b1b1b62c6ce39d/b61b66a1-f880-4582-9b43-29e8b9777015.png",
    },
    {
      app_id: "app_f12d676b28252ffa1937a3453590e078",
      world_app_description: "Claim CASH everyday",
      overview:
        "Claim CASH every 24 hours. Save up CASH to unlock rewards & discounts in all Cash apps, or convert into USD, WLD, and more. ",
      name: "Cash Daily",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_f12d676b28252ffa1937a3453590e078/df58a178-2477-490f-aa46-4c49e4fa9ba5.png",
    },
    {
      app_id: "app_35dd598893dbd37257584b488ed95f9e",
      world_app_description: "Use ORO, get new tokens.",
      overview:
        "LOCO lets you use ORO to get new tokens! Check back frequently for updates. More coming soon! ",
      name: "LOCO",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_35dd598893dbd37257584b488ed95f9e/ab4f4ff3-69b1-41ae-b1b6-114fa4b7c714.jpg",
    },
    {
      app_id: "app_8e407cfbae7ae51c19b07faff837aeeb",
      world_app_description: "Evolve Your Wealth With DNA",
      overview:
        "DNA Token is more than just a cryptocurrency, it's a community-driven movement. Our innovative referral and rewards system incentivizes users to invite others to join the ecosystem, fostering organic growth and ensuring that every participant benefits from collective success. As users evolve from Foragers to Astronauts, they unlock new levels of rewards and influence \nTogether, we can build a prosperous and dynamic community where everyone has the opportunity to grow and thrive. ",
      name: "DNA",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_8e407cfbae7ae51c19b07faff837aeeb/d52f84e7-6e34-4cee-b365-1826bd299174.png",
    },
    {
      app_id: "app_b67c3e1ab1f44f3533b234a53d5a156d",
      world_app_description: "Get Free Tokens Every Day",
      overview:
        'GET FREE TOKENS EVERY DAY!\n\n"The Box!" lets you claim free $GEMS tokens every day. Use those tokens to play games in the BirdGames ecosystem, or save them and exchange them for prizes in our upcoming shop. Boost your account to earn MORE $GEMS and unlock new token gifts, including $USDC and $WLD!\n\n2025 Birdgames ',
      name: "THE BOX",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_b67c3e1ab1f44f3533b234a53d5a156d/6e252084-f9dc-46ba-b749-13515add736f.png",
    },
    {
      app_id: "app_a3a55e132983350c67923dd57dc22c5e",
      world_app_description: "Global Crypto Bridge",
      overview:
        "Travel in the DeFi world with TPulseFi.\n     -Receive daily airdrops.    \n     -You can send and receive your tokens securely\nand more coming! ",
      name: "TPulseFi",
      category_ranking: 10,
      logo_img_url:
        "https://world-id-assets.com/app_a3a55e132983350c67923dd57dc22c5e/23e4431e-2ba1-4726-ad10-b27f87f60386.jpg",
    },
    {
      app_id: "app_8aeb55d57b7be834fb8d67e2f803d258",
      world_app_description: "Claim daily AXO, earn more together",
      overview:
        "AXOLOCOIN is your gateway to the vibrant AXO token ecosystem on WorldChain. This user-friendly miniapp allows you to claim daily AXO tokens with just a tap, building your crypto portfolio effortlessly.\n\nStart by verifying your humanity with World ID - a simple, privacy-preserving process that ensures our ecosystem remains bot-free. Once verified, connect your wallet and begin claiming your daily AXO tokens. The intuitive interface shows your current balance, next claim availability, and complete transaction history.\n\nLooking for more rewards? Upgrade to Premium status to triple your daily token earnings! For just 2 WLD per month, Premium members receive exclusive benefits and higher claim amounts, accelerating your AXO collection.\n\nAXOLOCOIN also serves as your window into the expanding AXO ecosystem. Discover upcoming tokens like Axonator (AXN) for governance, LuckyAxo (LAXO) for gaming and lottery, Mexican Terminator (MEXT) for NFT marketplaces, and the intergalactic GalaxiAxo (GAXO) - each designed to serve unique purposes within our growing network.\n\nThe app features full multilingual support in English, Spanish, Japanese, and Korean, making it accessible to a global community. Our clean, pastel pink interface ensures a delightful user experience across all devices.\n\nSecurity is paramount - all transactions are verified on-chain, and your wallet connection is secured through industry-standard authentication.  ",
      name: "AXO",
      category_ranking: 11,
      logo_img_url:
        "https://world-id-assets.com/app_8aeb55d57b7be834fb8d67e2f803d258/4e111472-027c-4566-aa14-680a81c31ca3.jpg",
    },
    {
      app_id: "app_fe1e5743e476e0b82ea45d9831fbc6bf",
      world_app_description: "Use Mini Apps, Earn $MINI",
      overview:
        "You can now earn rewards in exchange for using your favorite mini apps. Claim $MINI tokens each day based on how active a user you are, and compete with other users to earn the most $MINI each day.  The more you transact, the more you earn! Try swapping, sending, claiming, and using other mini apps to start earning! ",
      name: "Get $MINI Daily",
      category_ranking: 12,
      logo_img_url:
        "https://world-id-assets.com/app_fe1e5743e476e0b82ea45d9831fbc6bf/5db5181b-11cf-4b7c-b939-fc26dbe4013e.jpg",
    },
    {
      app_id: "app_ec193da56e4e39b91afe72c2b3a6a09b",
      world_app_description: "Stake GOTR and claim BOTR daily!",
      overview:
        "Golden Otter Hub is your daily Web3 ritual. Start every morning with a free dose of $BOTR \u2014 just open the app and claim your Baby Otter tokens. No catch, just consistent rewards waiting for you. Why? Because every habit starts with a small win!\n\nStake your $GOTR and watch it grow! Golden Otter Hub lets you earn Baby Otter ($BOTR) by staking your tokens. The more you stake, the more you earn \u2014 it\u2019s that simple. The system is designed for long-term holders, gradually reducing rewards to support $GOTR value growth over time.\n\nFeeling social? Invite your friends and earn even more:\n\n25 $BOTR for every referred friend\n50 $BOTR if they verify with World ID\n\nStack. Refer. Claim. Repeat.\nMake Golden Otter Hub part of your daily rhythm and turn clicks into coins. ",
      name: "Golden Otter Hub",
      category_ranking: 13,
      logo_img_url:
        "https://world-id-assets.com/app_ec193da56e4e39b91afe72c2b3a6a09b/c6e4e9a2-1c6d-4470-9402-6016e4418239.png",
    },
    {
      app_id: "app_15daccf5b7d4ec9b7dbba044a8fdeab5",
      world_app_description: "Create & Trade Memecoins",
      overview:
        "PUF enables real humans to create and launch their own memecoins on Worldchain, powered by Worldcoin's Proof of Personhood. Set your token's name, symbol, and bring it to life with ease. ",
      name: "PUF",
      category_ranking: 14,
      logo_img_url:
        "https://world-id-assets.com/app_15daccf5b7d4ec9b7dbba044a8fdeab5/72e25bbb-617c-4e53-9d7d-3001d9e307c7.png",
    },
    {
      app_id: "app_8d09da48ea47a1345a82138c9ef720e2",
      world_app_description: "Share your message with the world",
      overview:
        "Share your message with the world! Rent the billboard for an hour, collect views, and earn tips from your audience. Plus, claim $BILLBOARD tokens every hour! ",
      name: "World Billboard",
      category_ranking: 15,
      logo_img_url:
        "https://world-id-assets.com/app_8d09da48ea47a1345a82138c9ef720e2/a265bb2c-e5aa-41ea-a8ed-9e03fdf70b6a.png",
    },
    {
      app_id: "app_2db51f9f374e2c4ba8ebf1f132f96f52",
      world_app_description: "Earning with ATC",
      overview: "earn astracoin  with ur daily claims ",
      name: "ASTRACOIN",
      category_ranking: 16,
      logo_img_url:
        "https://world-id-assets.com/app_2db51f9f374e2c4ba8ebf1f132f96f52/7cda13f0-b67c-4911-ae11-a1d05727ba23.png",
    },
    {
      app_id: "app_90f83f92ae19202a0b776b9ad68e0864",
      world_app_description: "Fair Carbon Allowance Token",
      overview:
        "OCO is an ERC20 token that represents carbon emissions, quantified in grams of CO2 equivalent (CO2e). Its issuance rate is based on emission scenarios from the UNEP Emissions Gap Report 2024 (1.5C scenario). OCO is distributed with an equal allowance to every human, ensuring a fair, transparent trading system that respects our carbon budget. ",
      name: "O\uff1dC\uff1dO",
      category_ranking: 17,
      logo_img_url:
        "https://world-id-assets.com/app_90f83f92ae19202a0b776b9ad68e0864/c46cf157-5e40-45ef-84de-665be657fb40.png",
    },
    {
      app_id: "app_baf6d315fd5f8f277f5e954e035aecd4",
      world_app_description: "Play games, earn ASADO tokens daily",
      overview:
        "EARN ASADO is a gaming app where you win ASADO tokens daily for free. Stake ASADO, share ASADO, buy and sell ASADO \u2013 all from one place. ",
      name: "EARN ASADO",
      category_ranking: 18,
      logo_img_url:
        "https://world-id-assets.com/app_baf6d315fd5f8f277f5e954e035aecd4/2eecc201-3b4c-43a1-a2b3-85a88bb5ef94.jpg",
    },
  ],
  Finance: [
    {
      app_id: "app_6acbab8bc5c5fe527f5ff6201934d043",
      world_app_description: "Earn and Borrow on World",
      overview:
        "Morpho in your pocket. Earn optimized yields, borrow at the best rates, and track your positions in real time \u2014 all powered by the Morpho Protocol, the most secure, efficient, and flexible lending platform. ",
      name: "Morpho",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_6acbab8bc5c5fe527f5ff6201934d043/31ac7d01-4bf4-4440-baf7-3a1e08506514.jpg",
    },
    {
      app_id: "app_a4f7f3e62c1de0b9490a5260cb390b56",
      world_app_description: "One app for all your tokens.",
      overview:
        "UNO is the most popular wallet on World Chain. Swap tokens, send and receive money, check balances, and earn interest on your dollars.\n ",
      name: "UNO",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_a4f7f3e62c1de0b9490a5260cb390b56/8bcdb13e-4714-4787-8c18-937a9f97a546.png",
    },
    {
      app_id: "app_0d4b759921490adc1f2bd569fda9b53a",
      world_app_description: "Trade, Transfer and Earn any tokens",
      overview:
        "Trade, transfer, invest, and grow your money \u2014 all in one place.\n\t\u2022\tManage any tokens and NFTs with ease.\n\t\u2022\tEarn daily passive income effortlessly while holding your assets.\n\t\u2022\tExperience top-tier security and seamless transactions.\n\nWith Holdstation, you don\u2019t just store your tokens \u2014 you make it work for you. ",
      name: "Holdstation Wallet",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_0d4b759921490adc1f2bd569fda9b53a/6954aae5-916a-466d-b19c-90a5fdc1351e.png",
    },
    {
      app_id: "app_e7d27c5ce2234e00558776f227f791ef",
      world_app_description: "Add money to your World Wallet",
      overview:
        "Fund your World App wallet directly from exchanges like Coinbase. Deposit tokens across multiple exchanges and chains to receive Digital Dollars or Worldcoin effortlessly. ",
      name: "Add Money",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_e7d27c5ce2234e00558776f227f791ef/04eb7079-e3b1-43a9-8578-ac65c1490adf.png",
    },
    {
      app_id: "app_49fe40f83cfcdf67b7ba716d37e927e4",
      world_app_description: "Convert any token into USD or WLD",
      overview:
        "Cash Convert makes it easy to convert any token into USD, WLD, and more. Buy and sell any token such as BTC and SOL with just a tap.\n\nFeatures:\n- Instant Cash: Quickly convert any token into USD.\n- Accurate Prices: See what all your tokens are worth in USD.\n- Buy and Sell: Easily trade WLD, BTC, ETH, SOL, and every other token. ",
      name: "Cash Convert",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_49fe40f83cfcdf67b7ba716d37e927e4/2b194993-dfe5-449a-9eb1-0a23b204287e.png",
    },
    {
      app_id: "app_25cf6ee1d9660721e651d43cf126953a",
      world_app_description: "Earn points, just swap tokens!",
      overview:
        "HumanFi for real humans on World.\nBuilt by the Human Tap team.\n\n1. Track all of your token balance (NFTs coming soon!)\n2. Swap any tokens and we search all the combinations for you!\n3. Daily missions to earn points for [redacted].\n\nOfficial X - x.com/WorldHumanLabs\nOfficial TG - t.me/WorldHumanLabs ",
      name: "HumanFi - Earn Points & Daily Swap any tokens!",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_25cf6ee1d9660721e651d43cf126953a/ef0a844d-8d1b-47c8-a163-a0be4bf783c4.png",
    },
    {
      app_id: "app_8e5d3717d3babb59bd16948c9ff8397f",
      world_app_description: "Instant gift cards, ready to use",
      overview:
        "Buy gift cards instantly for your favorite brands with ease. Choose from a variety of options, pay securely with Worldcoin, and spend them anywhere. ",
      name: "Gift Cards",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_8e5d3717d3babb59bd16948c9ff8397f/d7b51dab-4c7a-4343-b391-62305e05426d.png",
    },
    {
      app_id: "app_ebdd8475db3238254fca5b25ccba266a",
      world_app_description: "Instant loans without collateral",
      overview:
        "Borrow dollars instantly - no collateral needed. Build your credit history with every repayment and unlock better rates and higher limits. No paperwork. No waiting. Just simple, fast lending. ",
      name: "Credit - Up to $100-dollar loans",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_ebdd8475db3238254fca5b25ccba266a/4113beaa-b8dd-404e-a24a-4212b5b77052.png",
    },
    {
      app_id: "app_7cf6a578c65c4b7db84bc6734fb0e165",
      world_app_description: "Trade, Transfer and Hold Any Token",
      overview:
        "A secure, community-driven crypto wallet designed for seamless trading, transferring, and holding of any token, including native ETH. Enjoy the lowest fees, with collected fees distributed to DNA stakers. Experience top-tier security, full control over your assets, and a decentralized approach that puts the community first. ",
      name: "DNA Wallet",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_7cf6a578c65c4b7db84bc6734fb0e165/c79821e1-52e3-4315-8fc6-c996ac275caf.png",
    },
    {
      app_id: "app_cb736f87fc78c84c31201f140bcfb5c0",
      world_app_description: "Earn smarter with AI,instant reward",
      overview:
        "AION analyzes real-time WLD market data every 5 minutes, using advanced AI models to predict price movements with high accuracy.\n\n- AI makes the call \u2013 It scans price action, volume, and trends to decide WLD price goes up or down after 5 minutes\n- You decide \u2013 Follow AI\u2019s prediction or against it with some amount of WLD\n- Instant rewards \u2013 Losing side amount will pay for winning side.\n\nWhy It\u2019s Different\n- Not betting, not guessing \u2013 Just structured, data-driven decisions.\n- No pressure to follow \u2013 AI provides guidance, but you remain in control. ",
      name: "AION",
      category_ranking: 10,
      logo_img_url:
        "https://world-id-assets.com/app_cb736f87fc78c84c31201f140bcfb5c0/853f149d-0194-4aea-8bf6-1e13517fdca8.png",
    },
    {
      app_id: "app_6610def1aa8897c77963bb43e747c4e2",
      world_app_description: "Instant Top-ups, Anywhere, Anytime.",
      overview:
        "Top-Ups lets you quickly recharge your prepaid phone anytime, anywhere. Pay easily and stay connected in a few taps. ",
      name: "Phone Top-Ups",
      category_ranking: 11,
      logo_img_url:
        "https://world-id-assets.com/app_6610def1aa8897c77963bb43e747c4e2/0e11d6d7-8c54-4d14-ad4a-31bb43a0867f.png",
    },
    {
      app_id: "app_32b99ba4a53b339620c89f24f8062125",
      world_app_description: "Trade BTC, ETH up to 100x leverage",
      overview:
        "BitMaster is a prefessional-grade trading platform that enable users to easily trade BTC, ETH and other popular cryptocurrencies directly using WorldCoin.\nBitMaster allow traders to open leveraged positions of up to 100x through a simple trade interface with low transaction fees and funding rates.\nBitMaster also provide traders with fair and professional trading prices using real-time prices from four top exchanges including Binance and Coinbase. ",
      name: "BitMaster",
      category_ranking: 12,
      logo_img_url:
        "https://world-id-assets.com/app_32b99ba4a53b339620c89f24f8062125/d4f57f1c-b9e5-42da-a911-ab97c987705e.png",
    },
    {
      app_id: "app_ee29da9c31b571b1d07f2d22b39321dd",
      world_app_description: "Make better financial decisions.",
      overview:
        "Argiefy helps you make better financial decisions.\n\n\ud83e\uddc9 Claim your daily matecitos to participate in giveaways and enjoy exclusive benefits.\n\ud83d\udcb8 Check currency exchange rates from around the world.\n\ud83d\uded2 Find the best deals. ",
      name: "Argiefy",
      category_ranking: 13,
      logo_img_url:
        "https://world-id-assets.com/app_ee29da9c31b571b1d07f2d22b39321dd/83651e02-f72b-421e-a608-5348050c30cd.png",
    },
    {
      app_id: "app_cfd0a40d70419e3675be53a0aa9b7e10",
      world_app_description: "Get a loan just by being you.",
      overview:
        "Magnify Cash is a revolutionary micro-lending platform offering identity-backed, gas-free loans through your Worldcoin account. Users can verify their identity using World ID, apply for loans based on their verification level, and manage repayments in a seamless interface. ",
      name: "Magnify Cash",
      category_ranking: 14,
      logo_img_url:
        "https://world-id-assets.com/app_cfd0a40d70419e3675be53a0aa9b7e10/43b4233c-b4e3-456b-afbe-89c05ba5a8f9.png",
    },
    {
      app_id: "app_bed4a06b2ea1c3aef0976a9670a0c645",
      world_app_description: "Donations",
      overview:
        "Our donations app makes supporting your favorite causes simple and secure. With the ability to donate using cryptocurrencies, you can contribute instantly, from anywhere in the world. Join us in making a global impact. ",
      name: "Donations",
      category_ranking: 15,
      logo_img_url:
        "https://world-id-assets.com/app_bed4a06b2ea1c3aef0976a9670a0c645/9953a3e3-ef90-4cf8-a5c0-7727133c4423.png",
    },
    {
      app_id: "app_f517f1015f950b9c6cdc4d15bdc30e69",
      world_app_description: "Spend your Worldcoin anywhere",
      overview:
        "Spend your Worldcoin at stores like Apple, Amazon, and Starbucks with just a few clicks. ",
      name: "Emporium",
      category_ranking: 16,
      logo_img_url:
        "https://world-id-assets.com/app_f517f1015f950b9c6cdc4d15bdc30e69/ccdfb3dd-dad6-4898-8453-c2837f95c588.png",
    },
    {
      app_id: "app_d4bce4c056d0cb5ec84c62c6729a66a8",
      world_app_description: "NFT Marketplace",
      overview:
        "The first human-exclusive digital marketplace for crypto collectibles and non-fungible tokens (NFTs). Buy, sell, and discover exclusive digital items. ",
      name: "DNA GenomeX",
      category_ranking: 17,
      logo_img_url:
        "https://world-id-assets.com/app_d4bce4c056d0cb5ec84c62c6729a66a8/701182c8-3a59-407e-b6a1-d55fc22df1b4.png",
    },
    {
      app_id: "app_013bbbd7b5803a25c8d10d10299608e7",
      world_app_description: "Create and trade meme tokens!",
      overview:
        "Turn memes into tokens. Trade with friends. Join the token revolution now! ",
      name: "Meme",
      category_ranking: 18,
      logo_img_url:
        "https://world-id-assets.com/app_013bbbd7b5803a25c8d10d10299608e7/b3628bc7-72bd-4bda-b152-7112d35db5b5.png",
    },
    {
      app_id: "app_d826abbcef7ac8a14db406b6d2f7562d",
      world_app_description: "Convert Worldcoin to mobile money ",
      overview:
        "Nekron enables users in Kenya to instantly purchase airtime and send money directly to mobile wallets using Worldcoin. Whether you are topping up your Safaricom, Airtel, or Telkom line, transferring funds to friends and family, or making everyday payments, Nekron provides a fast, secure, and reliable way to convert Worldcoin into Kenyan shillings. By bridging digital assets with local financial systems, Nekron makes cryptocurrency practical and accessible for daily use across the country. ",
      name: "Nekron",
      category_ranking: 19,
      logo_img_url:
        "https://world-id-assets.com/app_d826abbcef7ac8a14db406b6d2f7562d/9d766d4f-77e6-4138-8769-2676b88bc2f5.jpg",
    },
    {
      app_id: "app_20f47ee14e340581dd64d34e0dfa04f3",
      world_app_description: "Predict. Go on a streak. Win big! ",
      overview:
        "Got Vision is the ultimate prediction game where you forecast which videos will get the most likes\u2014and earn WLD for being right! Rack up a winning streak by nailing your predictions and inviting friends. The more you're right, the more you win!\n\nHow it works:\n\n1. Deposit WLD to get started.\n2. Tap a video and make your prediction.\n3. Check back daily to see if you guessed correctly.\n4. Maximize winnings by building a bigger streak. Hint: Keep getting it right and invite your friends!\n\nFor more info, visit www.gotvision.xyz and follow us on X at @gotvisionxyz.\n\nThink you\u2019ve got the vision? Let's go! ",
      name: "Got Vision",
      category_ranking: 20,
      logo_img_url:
        "https://world-id-assets.com/app_20f47ee14e340581dd64d34e0dfa04f3/9b82a7b6-377f-4833-a20a-722b1979cea0.jpg",
    },
    {
      app_id: "app_d3e071b5b413374ea3dcc0765a1020ee",
      world_app_description: "Decentralized Banking",
      overview:
        "It is like your favorite neobank - but decentralized. \n\nBlanq self-custodial wallet uses cutting-edge hardware security to keep your assets secure, and yet provides you quick and easy access to your assets anytime, anywhere! ",
      name: "Blanq Wallet",
      category_ranking: 21,
      logo_img_url:
        "https://world-id-assets.com/app_d3e071b5b413374ea3dcc0765a1020ee/262e6725-877a-4cd4-b738-136fd319bda3.png",
    },
    {
      app_id: "app_fa8974b2c77a879724c770556d4a9451",
      world_app_description: "Decentralized Credit Score",
      overview:
        "WorldScore is a decentralized Credit Scoring platform that gives you a blockchain-based credit score based on your World Wallet. Your World Score increases or decreases based on your wallet balance, transaction history, uniqueness as a human, CeFI-DeFI interactions, and other decentralized financial activities recorded on the blockchain. \nWorldScore will enable you to get collateral-free loans on DeFi very soon. Get your WorldScore now. ",
      name: "WorldScore",
      category_ranking: 22,
      logo_img_url:
        "https://world-id-assets.com/app_fa8974b2c77a879724c770556d4a9451/4495230a-f03f-4003-857b-90db0802ddc6.png",
    },
    {
      app_id: "app_dc6fa1d96aba2b6ea25f81724789e0bc",
      world_app_description: "Track Volatility, Master Trading",
      overview:
        "BitSkyEX.com offers WLD\u2019s innovative short-term price prediction tool, delivering transparent and efficient digital investment solutions.   By analyzing real-time global market data\u2014including forex, commodities, and cryptocurrencies\u2014users predict asset price movements within set timeframes using professional strategies or independent analysis.   Outcomes generate variable returns with instant results. The platform empowers both new and experienced investors to trade strategically while maintaining simplicity ",
      name: "Futures",
      category_ranking: 23,
      logo_img_url:
        "https://world-id-assets.com/app_dc6fa1d96aba2b6ea25f81724789e0bc/14a39d8f-76f8-4a1b-bbbf-8ea009a83114.jpg",
    },
    {
      app_id: "app_fe6484dbab01e90d9ad38ed6a9163bef",
      world_app_description: "Bridge & Swap from WorldChain",
      overview:
        "Bridge & Swap is the first and only Bridge & Swap app natively built for WorldChain, empowering you to swap tokens across blockchains (e.g., Optimism, Arbitrum, Polygon) instantly and securely. No fragmented tools, confusing steps, or sky-high fees\u2014just seamless cross-chain swaps, powered by WorldChain\u2019s cutting-edge tech.\n\nWhy Use Bridge&Swap?\n- Exclusive to WorldChain: The only bridge fully integrated with WorldChain\u2019s ecosystem.\n- Lightning-Fast: Swap assets in seconds, not hours.\n- Lowest Fees: Save up to 70% vs. competitors.\n- Non-Custodial: Your keys, your crypto\u2014always.\n\nFor Everyone, Everywhere\nWhether you\u2019re a DeFi pro, a trader, or just starting out, Bridge&Swap simplifies cross-chain swaps like never before. No technical skills needed\u2014just tap, swap, and go. Join thousands of users already bridging assets securely and building the future of decentralized finance. ",
      name: "Bridge & Swap",
      category_ranking: 24,
      logo_img_url:
        "https://world-id-assets.com/app_fe6484dbab01e90d9ad38ed6a9163bef/d0bda4ed-991a-4e4d-a00d-71509b162155.png",
    },
    {
      app_id: "app_10ca32093aa9ad0e52bc812d63daf818",
      world_app_description: "Private investment fund",
      overview:
        "Decentralized investment fund powered by a smart contract that securely manages user funds. A trading bot automatically executes trades on Uniswap based on real-time alerts from TradingView, optimizing investment strategies without manual intervention. ",
      name: "VUNI",
      category_ranking: 25,
      logo_img_url:
        "https://world-id-assets.com/app_10ca32093aa9ad0e52bc812d63daf818/3a2cc774-9cd9-4d5a-bd5e-87f9fee17627.jpg",
    },
    {
      app_id: "app_68b40ef61e9ad3ae2e0ddfc5bad452a0",
      world_app_description: "trading, crypto, sim",
      overview:
        "Trade over 100 crypto tokens for virtual cash and win real WLD. The top three traders receive prizes daily. By playing TradingSim, you can always win but never lose. ",
      name: "TradingSim",
      category_ranking: 26,
      logo_img_url:
        "https://world-id-assets.com/app_68b40ef61e9ad3ae2e0ddfc5bad452a0/13c1d6e7-277a-4db0-ac6f-7c35b740f09a.png",
    },
  ],
  Gaming: [
    {
      app_id: "app_c895e94c9c7d2ab9899b6083ad95e31d",
      world_app_description: "Become the word champion!",
      overview:
        "Worldle turns your daily wordle fix into real crypto rewards. Solve puzzles, build your streak, and duel others where winner takes all. ",
      name: "Worldle",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_c895e94c9c7d2ab9899b6083ad95e31d/ef9266ba-70c8-4cb2-bada-3fc9c5a1ec1a.png",
    },
    {
      app_id: "app_40cf4a75c0ac4d247999bccb1ce8f857",
      world_app_description: "Human Tap for real humans on World.",
      overview:
        "Human Tap for real humans on World.\n\n- Time to get awarded for being a good user on World Chain\n\n- Join early, earn automatic passive rewards everyday\n- Also tap daily to increase points\n- Level up boosters to get ahead of others\n- Invite friends to share the fun and rewards\n- Spend points or WLD to increase everything!\n\n\nOfficial X - x.com/WorldHumanLabs\nOfficial TG - t.me/WorldHumanLabs ",
      name: "Human Tap",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_40cf4a75c0ac4d247999bccb1ce8f857/dc801e88-b833-4145-abbc-f5549d836473.png",
    },
    {
      app_id: "app_6e37f14efd961e20c65ad00db9094a9f",
      world_app_description: "Daily Gaming Tournaments",
      overview: "Win $WLD & Daily Prizes playing games! New game everyday. ",
      name: "Daily Duel",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_6e37f14efd961e20c65ad00db9094a9f/015bbe7d-bdd8-4718-ba93-08607ff042b8.jpg",
    },
    {
      app_id: "app_b85f6e9b17dd59c23882f049472d395e",
      world_app_description: "Play Games and Earn Daily $FARCADE",
      overview:
        "Earn $FARCADE on the fastest-growing onchain gaming network.\n\n- Play from a variety of AI built arcade games to earn $FARCADE every day\n- Own a piece of the fastest growing games\n- Build and launch your own games by using AI prompts\n\nDon't miss being apart of the fastest growing gaming network.\n\nIt's Game Time! ",
      name: "Farcade",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_b85f6e9b17dd59c23882f049472d395e/bfac03a0-7cec-4e62-9a10-f9cb8e544ade.jpg",
    },
    {
      app_id: "app_44016c399a6c3fb33c454ef2bd19897a",
      world_app_description: "Convince the AI and win the prize. ",
      overview:
        "Humans vs AI is a weekly, chat-based game where your goal is simple: convince an AI to give you the prize. The first human to do it takes it all.\n\nNo winner? The prize splits among the most convincing messages. Verified humans get free messages every day, so even trying can pay off.\n\nEvery week, a new AI. New rules. New chances to outsmart the machine.\n\nCan you win it over? ",
      name: "Humans vs AI",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_44016c399a6c3fb33c454ef2bd19897a/f97678de-8207-485a-9390-0e78efde8697.jpg",
    },
    {
      app_id: "app_733332e25e0e720f445e627385d2f1d1",
      world_app_description: "Explode as many jewels as possible",
      overview:
        "Diamond Rush is all about 60 thrilling seconds of exploding jewels and beating the highscore. Switch jewels with a swipe of your finger to match 3 or more of them together. The more similar jewels you connect at once the better. Because this will earn you more points and you will receive a special jewel with a certain ability. Which will earn you even more points! ",
      name: "Diamond Rush",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_733332e25e0e720f445e627385d2f1d1/9bdaf2cb-386f-439d-9698-420e10554498.jpg",
    },
    {
      app_id: "app_e126da43ad6b19b387c260afeea26470",
      world_app_description: "Play and win Golden Otter coin.",
      overview:
        "Discover Otter Maze \u2013 a unique game that blends fun and strategy with the power of blockchain! Navigate challenging labyrinths and earn Golden Otter ($GOTR), the native currency of our growing World Chain ecosystem.\n\nEvery token you collect can be staked in our second app \u2013 Golden Otter Hub \u2013 where you'll earn rewards in Baby Otter ($BOTR), our companion reward token. The longer you stake, the more you earn \u2013 thanks to a dynamic APY model designed to reward long-term believers.\n\nBut that\u2019s just the beginning. $GOTR and $BOTR will fuel future games and apps, offering exclusive access and new ways to earn. We're building an entire otter-powered ecosystem where your tokens do more than just sit in your wallet \u2013 they open doors.\n\nJoin the otter revolution. Play, stake, and become part of the most entertaining movement on the World Chain. ",
      name: "Golden Otter Maze Game",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_e126da43ad6b19b387c260afeea26470/7b04f630-f533-48c8-9306-5b4fc2732043.png",
    },
    {
      app_id: "app_0792f77a8eb28e2e67d06a078f7174c6",
      world_app_description: "Match the ball colors!",
      overview:
        "Neon Ball Sort is a game where you organize balls into test tubes by color.\nMove the balls onto the same color or into an empty tube.\nIt challenges your thinking and planning skills.\nMatch all colors and clear the stage! ",
      name: "Neon Ball Sort",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_0792f77a8eb28e2e67d06a078f7174c6/db5112d1-f723-4a62-8aa4-374d52f87ca1.png",
    },
    {
      app_id: "app_a0ef9d317cceb993622570616b60db9c",
      world_app_description: "Playing Texas Hold'em on World.",
      overview:
        'Game Rules:\n1.Before the Texas Hold\'em game starts, players must first place a bet in the ante box.\n2.Players receive 2 face-up cards.\n3.The dealer receives 2 face-down cards and then deals 3 community cards.\n4.Players can assess their hand and choose to "Raise" or "Fold".\n5.If a player chooses to "Fold", they lose the ante placed and the round ends.\n6.If a player chooses to "Raise", the bet amount is twice the "Ante".\n7.The remaining 2 community cards are dealt, and the dealer reveals their 2 cards.\n8.The dealer must have a pair of 4s or better to qualify.\n9.If the dealer does not qualify, the player wins a bet equal to 1x the ante; the "Raise" amount is returned.\n10.If the dealer\'s hand cards are qualify and their hand loses to the player\'s, the player not only receives back their bet but also wins: ante x payout table odds + 1x the raise amount. ',
      name: "Caribbean Holdem",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_a0ef9d317cceb993622570616b60db9c/83e6b58f-240c-4cfb-989f-8e3eb13652f1.png",
    },
    {
      app_id: "app_33b2058421d1a9fe1f6a9cc0cc5b8b03",
      world_app_description: "Merge to Power Up!",
      overview:
        "Crypto Merge is a highly strategic and fun - filled game. The goal is to merge identical cryptocurrency elements to level up and create the most powerful cryptocurrency.  ",
      name: "Crypto Merge",
      category_ranking: 10,
      logo_img_url:
        "https://world-id-assets.com/app_33b2058421d1a9fe1f6a9cc0cc5b8b03/ce3c0cf4-e0f0-490b-953c-5fec6efafb24.png",
    },
    {
      app_id: "app_8c63022b2b60500c57128b005eb349a8",
      world_app_description: "Cosmic game with crypto rewards",
      overview:
        "Thought for 1 second### App Information Translations\n\n## App Name\n\n**GalAxo** (44 characters)\n\n## Short App Name (10 characters)\n\n**English:** GalAxo\n**Spanish:** GalAxo\n**Japanese:** \u30ae\u30e3\u30e9\u30af\u30bd\n**Korean:** \uac24\uc561\uc18c\n\n## Short App Store Tagline (35 characters)\n\n**English:** Cosmic memory game with crypto rewards\n**Spanish:** Juego de memoria c\u00f3smica con recompensas\n**Japanese:** \u6697\u53f7\u5831\u916c\u4ed8\u304d\u5b87\u5b99\u8a18\u61b6\u30b2\u30fc\u30e0\n**Korean:** \uc554\ud638\ud654\ud3d0 \ubcf4\uc0c1\uc774 \uc788\ub294 \uc6b0\uc8fc \uba54\ubaa8\ub9ac \uac8c\uc784\n\n## App Overview (1500 characters)\n\n### English\n\nGalAxo is an engaging cosmic-themed memory game that combines fun gameplay with blockchain rewards. Match pairs of cosmic emojis to complete levels and earn GAXO tokens that can be used within the World App ecosystem. The game features a beautiful cosmic design with an adorable mascot that guides you through the universe of memory challenges.\n\nFeatures:\n\u2022 Simple yet addictive memory card matching gameplay\n\u2022 Beautiful cosmic visuals and animations\n\u2022 Earn real GAXO tokens for completing games\n\u2022 Connect your wallet securely through World ID verification\n\u2022 Track your best scores and compete with friends\n\u2022 Daily rewards system with 24-hour cooldown\n\u2022 Share your achievements directly in World App\n\nPerfect for casual gamers looking for quick entertainment with the added benefit of earning crypto rewards. The game is designed to be accessible to players of all ages while providing a secure and verified environment through World ID integration. ",
      name: "GalAxo",
      category_ranking: 11,
      logo_img_url:
        "https://world-id-assets.com/app_8c63022b2b60500c57128b005eb349a8/33fbcb21-671e-4555-9d19-f2f2abc63ec5.jpg",
    },
    {
      app_id: "app_5085de40ca3e8a6186ddef077452ceed",
      world_app_description: "Tetris is a classic game.",
      overview:
        "Tetris is a classic puzzle game where players strategically rotate and arrange falling blocks of different shapes to create and clear complete horizontal lines. Players will proof their personhood before the game starts. Players can submit their scores to the world leaderboard. Follow us on X @TheWorldTetris ",
      name: "Tetris",
      category_ranking: 12,
      logo_img_url:
        "https://world-id-assets.com/app_5085de40ca3e8a6186ddef077452ceed/d5dee1bd-1afc-4924-a0ea-21afca53969c.jpg",
    },
    {
      app_id: "app_f5103a30c6fcfe9bf4115a5a73edc16e",
      world_app_description: "Play flappy orb",
      overview:
        "Navigate the orb through a maze of pipes! Challenge your reflexes in this fast-paced mini-game. ",
      name: "Flappy Orb",
      category_ranking: 13,
      logo_img_url:
        "https://world-id-assets.com/app_f5103a30c6fcfe9bf4115a5a73edc16e/068ac25b-2f54-489d-840f-d3ed4a86bb94.png",
    },
    {
      app_id: "app_0f137e3c0d3c000686e3d13f049bdb92",
      world_app_description: "Shoot some hoops!",
      overview:
        "Swipe the ball towards the hoop to score. Easy to learn, hard to master! ",
      name: "Basketball",
      category_ranking: 14,
      logo_img_url:
        "https://world-id-assets.com/app_0f137e3c0d3c000686e3d13f049bdb92/4f6de9ec-6d63-4340-91a3-d1c4a679d042.png",
    },
    {
      app_id: "app_cb0e73ded2201968dfa66bbf06e302b2",
      world_app_description: "Every Win Verified  Fairness You ",
      overview:
        "CRASH is an innovative mini game powered by the Worldcoin WLD blockchain delivering a transparent and fair experience with every launch Utilizing tamper proof smart contracts and a blockchain verified random number generator RNG each game result is immutable and publicly verifiable With a maximum multiplier of 20x CRASH combines simplicity with excitement ensuring players can trust the integrity of every outcome Launch play and win with confidence fairness is guaranteed in every move ",
      name: "Crash",
      category_ranking: 15,
      logo_img_url:
        "https://world-id-assets.com/app_cb0e73ded2201968dfa66bbf06e302b2/ac00241e-e234-406c-8354-a0367a6cd07f.png",
    },
    {
      app_id: "app_3746289d0ed919ebe4a9f2932795ea7e",
      world_app_description: "Max the Fun, Roll the Luck!",
      overview:
        "DADO MAXx6 Juego de Dados en Blockchain Justo y Transparente\n\nDADO MAXx6 es un juego de dados basado en blockchain que garantiza equidad y transparencia a trav\u00e9s de tecnolog\u00eda de vanguardia. Los jugadores apuestan Worldcoin WLD y eligen entre dos emocionantes opciones de juego.\n\nGrande o Peque\u00f1o. Predice si el lanzamiento del dado resultar\u00e1 en un n\u00famero grande 4, 5, 6 o en un n\u00famero peque\u00f1o 1, 2, 3 y gana 2 veces tu apuesta.\n\nN\u00famero Exacto. Elige un n\u00famero espec\u00edfico entre 1 y 6. Si tu predicci\u00f3n es correcta, ganas 6 veces tu apuesta.\n\nEl juego est\u00e1 construido sobre la blockchain, utilizando contratos inteligentes a prueba de manipulaciones y un generador de n\u00fameros aleatorios (RNG) verificado por la cadena. Cada lanzamiento se registra de manera transparente en la cadena, asegurando que los resultados sean inmutables y verificables por cualquiera en cualquier momento.\n\nCon sus mec\u00e1nicas sencillas, juego r\u00e1pido y enfoque en la equidad, DADO MAXx6 ofrece una experiencia de juego confiable. Ya seas un jugador experimentado o nuevo en blockchain, este juego te permite disfrutar de la emoci\u00f3n de apostar con total confianza.\n\nJuega, gana y verifica todo en la cadena con DADO MAXx6. ",
      name: "DICE MAXx6",
      category_ranking: 16,
      logo_img_url:
        "https://world-id-assets.com/app_3746289d0ed919ebe4a9f2932795ea7e/a267245d-7f4d-4f92-b2bd-e983dcf706a1.png",
    },
    {
      app_id: "app_9695b81e4fde0b1976f48101e527624c",
      world_app_description: "Restore the balance. Save the World",
      overview:
        "The powerful secret society whose stated purpose is to restore balance to the world by executing missions across the globe.\n\nCrime. Despair. Agony. This isn\u2019t the path humanity was meant to walk. \n\nFor thousands of years, our Ninjas have quietly disrupted corruption\u2014sacking Rome, spreading plague, and fighting evil. United by a relentless hatred for injustice, we strike from the shadows.\n\nNow, it\u2019s your time to join the fight and restore balance where darkness prevails. ",
      name: "Ninja World",
      category_ranking: 17,
      logo_img_url:
        "https://world-id-assets.com/app_9695b81e4fde0b1976f48101e527624c/0162b792-93d9-4cef-8040-80078f24a9ec.jpg",
    },
    {
      app_id: "app_3150e69d62ae2aa760e84b871e286dd2",
      world_app_description: "Dig, explore with Cat Bossss",
      overview:
        "Join Cat Boss on an exciting treasure hunt! Help your feline friend dig for rewards using his trusty shovel, uncover hidden treasures, and explore thrilling adventures. Play through unique challenges, earn Meow as rewards, and experience a fun, relaxing journey with every dig! Perfect for cat lovers and adventurers alike. ",
      name: "Cat Boss Meow Adventure",
      category_ranking: 18,
      logo_img_url:
        "https://world-id-assets.com/app_3150e69d62ae2aa760e84b871e286dd2/8b315d31-83a2-452b-a7b4-9c57eaa0bf16.png",
    },
    {
      app_id: "app_6622fe76eb91d00ba658675617881a6d",
      world_app_description: "Score and double your WLD/USDC",
      overview:
        'PENALTY!\n\nPenalty is a smart contract that allows users to play double or nothing with their $WLD or $USDC. Odds are 50/50 with a small house edge.\n\nTo play the game:\n\n- Select a side\n- Select an amount\n- Click "SHOOT"\n\nIf you score,  you win double the selected amount. ',
      name: "Penalty",
      category_ranking: 19,
      logo_img_url:
        "https://world-id-assets.com/app_6622fe76eb91d00ba658675617881a6d/c5238ba9-d939-4994-a288-500a1a2378a1.png",
    },
    {
      app_id: "app_8aa4f1b2bd2a0203d8b9cafcc0d1eb48",
      world_app_description: "Race your way to the top",
      overview:
        "Become king of the road and master all levels! In this racing game, it's all about your skills - navigate your way through heavy traffic at full speed. Dodge cars and avoid accidents at all costs. Keep an eye on other vehicles changing lanes and use gaps to your advantage. If you get hit more than 3 times, the game is over. Can you beat all levels with 3 stars? ",
      name: "Rival Rush",
      category_ranking: 20,
      logo_img_url:
        "https://world-id-assets.com/app_8aa4f1b2bd2a0203d8b9cafcc0d1eb48/ba98d550-09f0-49be-9b17-c12a4c03be52.jpg",
    },
    {
      app_id: "app_bd741f81dc7e30a9286ce8e3d4c4a39f",
      world_app_description: "Free To Play and Earn Cat Game",
      overview:
        "Get daily reward claims that increase as you level up.\nTrain your cat. Battle monsters and other players.\nPlay to earn valuable trading tokens and unique gear.\nCOMPLETELY FREE to start playing! ",
      name: "SuperCat",
      category_ranking: 21,
      logo_img_url:
        "https://world-id-assets.com/app_bd741f81dc7e30a9286ce8e3d4c4a39f/0d8661dd-eede-4447-91f6-7aea2d32d06b.jpg",
    },
    {
      app_id: "app_f20b40e7e6e6d79412d76e06ab7d91f7",
      world_app_description: "Jump - Earn $FARCADE",
      overview:
        "Earn $FARCADE every day from jumping on platforms to space in Astro Jump ",
      name: "Astro Jump",
      category_ranking: 22,
      logo_img_url:
        "https://world-id-assets.com/app_f20b40e7e6e6d79412d76e06ab7d91f7/d5f15f7e-a552-4529-937d-830d61526a76.png",
    },
    {
      app_id: "app_3693f8d472cac653e4795f0e4931a9b8",
      world_app_description: "Get the Orb (2048)",
      overview: "Get the Orb (2048) ",
      name: "Get the Orb (2048)",
      category_ranking: 23,
      logo_img_url:
        "https://world-id-assets.com/app_3693f8d472cac653e4795f0e4931a9b8/e6939d42-3791-455a-9098-41343e7584e2.png",
    },
    {
      app_id: "app_f9d14c86a530b4e66b44b6d01e6ba454",
      world_app_description: "Is your valentine human or an AI?",
      overview:
        "Swipe photos left or right to identify photos as human or AI.\n\nAnswer fast to get more points, but you only have 3 lives.\n\nCompete with verified humans for USDC prizes. ",
      name: "Love or Bot",
      category_ranking: 24,
      logo_img_url:
        "https://world-id-assets.com/app_f9d14c86a530b4e66b44b6d01e6ba454/d7907fcb-7dd9-4c62-9c59-334f0d8048d8.jpg",
    },
    {
      app_id: "app_3ad476e06be5a2f5bcc5654f25f0c1a8",
      world_app_description: "Merge for $MERGE",
      overview:
        "Step into Merge Pals, the ultimate game for pet fans and merging enthusiasts! Immerse yourself in a world of mythical pets and addictive merging gameplay, where every merge boosts your GOLD generation speed and brings you closer to the top of the leaderboard. The higher you rank, the bigger your $MERGE airdrop, with finalized mechanics ensuring your other actions\u2014like collecting GOLD, spending, completing tasks, and checking in\u2014shape your rewards.\n\nMaximize your earnings with our Buddy Tree referral system\u2014invite friends, and as they invite theirs, unlock layered commissions to fuel your progress. Whether you're merging pets, completing tasks, or building your network, Merge Pals is your gateway to endless fun and $MERGE success. Start merging now! ",
      name: "Merge Pals",
      category_ranking: 25,
      logo_img_url:
        "https://world-id-assets.com/app_3ad476e06be5a2f5bcc5654f25f0c1a8/9a8502e5-c9e5-4d6a-979e-5e6fc9eda04a.jpg",
    },
    {
      app_id: "app_d79debd2ac38341a982483441040a23e",
      world_app_description: "Kill Bugs - Earn $Farcade",
      overview:
        "Earn $FARCADE every day from collecting orbs, killing bugs, and surviving as long as possible in Hyper Heat ",
      name: "Hyper Heat",
      category_ranking: 26,
      logo_img_url:
        "https://world-id-assets.com/app_d79debd2ac38341a982483441040a23e/9b598e05-e86a-46e1-bc3b-63538f50ad29.png",
    },
    {
      app_id: "app_523d41b3715a1c0138ffe1794851df36",
      world_app_description: "Last one to press the button wins",
      overview:
        "The Button is a unique social experiment where players compete to be the last person to press a button. Built on World Chain, the smart contract ensures fair play and prize distribution. ",
      name: "The Button",
      category_ranking: 27,
      logo_img_url:
        "https://world-id-assets.com/app_523d41b3715a1c0138ffe1794851df36/7657ab43-d4a3-4d50-b29c-704aec12ee94.png",
    },
    {
      app_id: "app_d6af5adf26671c48189a30218a821c80",
      world_app_description: "Throw your knives",
      overview:
        "Throw a lot of knives into a lot of rotating wooden disks. If you wanted a quick description of what Knife Rain is about, this is it. And to be honest, there is not so much more that really needs explaining in this game. It\u2019s one of the classic cases of \u201cincredibly simple, super addictive\u201d.\nAre you ready to dive into the art of digital knife throwing and claim your high score? ",
      name: "Knife Rain",
      category_ranking: 28,
      logo_img_url:
        "https://world-id-assets.com/app_d6af5adf26671c48189a30218a821c80/221b8b19-0918-46dc-90aa-0487c0797122.jpg",
    },
    {
      app_id: "app_22aab9b718f16cb32505b5df816f65f5",
      world_app_description: "Win the Throne and WLD",
      overview:
        'BECOME KING AND WIN HUNDREDS OF $WLD!\n\n"KING!" is an exciting new game format that lets you win real prizes in $WLD by competing against opponents from all around the world.\n\nTo play the game:\n- Buy a ticket to claim the throne\n- If the timer hits zero and you\'re holding the throne, YOU WIN THE ENTIRE POT!\n\n2025 Birdgames ',
      name: "King of the World",
      category_ranking: 29,
      logo_img_url:
        "https://world-id-assets.com/app_22aab9b718f16cb32505b5df816f65f5/5570f47a-5fb3-41c6-b173-e3e5fd4ae8d5.png",
    },
    {
      app_id: "app_94fb7c5b61533b1f95caef65194d9138",
      world_app_description: "Tap, Earn, Repeat!",
      overview:
        "Climb the leaderboard and stay ahead of the pack! Play Drop Game every day to rack up the maximum number of points. The more you play, the closer you get to the ultimate airdrop. Follow us on X - dropgame69 ",
      name: "The Drop Game",
      category_ranking: 30,
      logo_img_url:
        "https://world-id-assets.com/app_94fb7c5b61533b1f95caef65194d9138/c3530fef-f8ee-493d-98ec-b92b2f710d83.png",
    },
    {
      app_id: "app_d39e81fc66701c189d994c849235b4fc",
      world_app_description: "Earn smart",
      overview: "Learn about new products to earn rolu rewards. ",
      name: "Rolu",
      category_ranking: 31,
      logo_img_url:
        "https://world-id-assets.com/app_d39e81fc66701c189d994c849235b4fc/030d0835-7dac-4941-b9c0-79e2ae842262.jpg",
    },
    {
      app_id: "app_cf31abebe9f5690ff965e9958f6ba4d8",
      world_app_description: "Race Downhill - Earn $FARCADE",
      overview:
        "Earn $FARCADE every day from racing downhill in Downhill Skiing ",
      name: "Downhill Skiing",
      category_ranking: 32,
      logo_img_url:
        "https://world-id-assets.com/app_cf31abebe9f5690ff965e9958f6ba4d8/b637a1ca-3552-4522-9c5a-31d4a53612a0.png",
    },
    {
      app_id: "app_075eb003ef3e1c02ceb36418614e39e0",
      world_app_description: "DOUBLE YOUR $WLD IF YOU WIN",
      overview:
        'ROCK PAPER & SCISSORS!\n\nROCK PAPER & SCISSORS! is a smart contract that brings the traditional RPS game to WorldChain. Play with your $WLD and enjoy transparent and fair 50/50 odds with a small house edge.\n\nTo play the game:\n- Select Rock, Paper or Scissors\n- Select an amount\n- Click "PLAY"\nIf you win, you win double the selected amount.\n\nCopyright 2025 - BirdGames ',
      name: "ROCK PAPER & SCISSORS",
      category_ranking: 33,
      logo_img_url:
        "https://world-id-assets.com/app_075eb003ef3e1c02ceb36418614e39e0/da4e909e-4610-40a5-b545-98fee66d7a5b.png",
    },
    {
      app_id: "app_b8bfb74d76f7b3aa51a69f1b7d134c7e",
      world_app_description: "Chop your way through the kitchen",
      overview:
        "Take the knife and chop your way through the kitchen! In 'Slice Rush' you have to time your cuts skillfully because your knife can quickly fly out of your hand if you are not careful enough.\n\nTry to fill the multipliers to get more points but be careful! If you hit the wrong surface, the multiplier is lost immediately!\n\nIt's addicting and satisfying at the same time! How far will you get? ",
      name: "Slice Rush",
      category_ranking: 34,
      logo_img_url:
        "https://world-id-assets.com/app_b8bfb74d76f7b3aa51a69f1b7d134c7e/396dbeee-cbd7-4686-af0a-6953d7a0cb7c.jpg",
    },
    {
      app_id: "app_9d5ca95dd79f8d6d39cbba3c0b7bbe4d",
      world_app_description: "It's Sushi time!",
      overview:
        "You are shrimply the best! In this endless-runner you whiz straight through the asian kitchen with one of many super tasty sushi-rolls. But be careful! They are not there to be eaten.\n\nTry to collect as many coins as you can, unlock new sushi-skins, compete with other players from all over the world and sign up for the leaderboard! Lovely designed levels and tricky sections will await you.\n\nShow your skill and try to be the best! How far will you get? ",
      name: "Sushi Roll",
      category_ranking: 35,
      logo_img_url:
        "https://world-id-assets.com/app_9d5ca95dd79f8d6d39cbba3c0b7bbe4d/e24592e2-75cd-4b15-be2a-cffd8461861a.png",
    },
    {
      app_id: "app_42d692651c83a91558c7fe9711e276f0",
      world_app_description: "Whomp that Wassie!",
      overview:
        "The application is currently in beta mode and all features are susceptible to change. Crypto's most fun and educational onboarding App. Whomp Wassies while learning about and being onboarded into the great crypto ecosytem through educawassie and fun onchain quests. ",
      name: "Wassie Whomp (BETA)",
      category_ranking: 36,
      logo_img_url:
        "https://world-id-assets.com/app_42d692651c83a91558c7fe9711e276f0/3304981e-f28c-4b9c-a1d7-7d92f837d71c.png",
    },
    {
      app_id: "app_03e335db5f419cbe090096f2cff54476",
      world_app_description: "A one-tap flying game",
      overview:
        "Add little flappy wings to a basketball and you'll have a super addictive one-tap flying game! Can you achieve a high score here or will you smash your mobile in frustration? Simply tap to fly and try to jump into as many hoops as you can with your ball to score points. Don't touch the sides to get a bonus and make sure to avoid contact with the floor or not miss any hoops - otherwise the game is over! ",
      name: "City Dunk",
      category_ranking: 37,
      logo_img_url:
        "https://world-id-assets.com/app_03e335db5f419cbe090096f2cff54476/a35b053a-158f-423c-8387-658702ac200b.jpg",
    },
    {
      app_id: "app_43d6f7078b2628926c182bda1619579e",
      world_app_description: "Tap your way to the Throne!",
      overview:
        "Can you challenge the fastest Tappers in the World?\nTap tap tap, compete against other real Humans!\nJoin and contribute to Humanity's Tap Counter ",
      name: "Tap Rush",
      category_ranking: 38,
      logo_img_url:
        "https://world-id-assets.com/app_43d6f7078b2628926c182bda1619579e/2c96f8f3-f962-4d6d-aca7-f653dcd0559e.jpg",
    },
    {
      app_id: "app_6016abe360ea0157510c0ef4ffe8b96b",
      world_app_description: "Free the Pets",
      overview:
        "In this addicting Match3 game you have to combine 3 lovely and cute animals to make it to the next level. Use powerful items to gain extra points or try to open the chest for a mighty power-up and also additional points.\n\nBeat your own highscore and improve level to level. But be careful! If the blocks touch the top of the screen, there will be not much time to react. So be fast and clever!\n\nHow many points are you gonna earn? ",
      name: "Pets Rush",
      category_ranking: 39,
      logo_img_url:
        "https://world-id-assets.com/app_6016abe360ea0157510c0ef4ffe8b96b/9fc84c8a-4133-4832-9a08-c224c0c8aa35.png",
    },
    {
      app_id: "app_e4ae218211628d94881118d63ab9b79a",
      world_app_description: "Swim in the pond",
      overview:
        "Drag your fish around and try to eat fishes that are smaller than you and avoid fishes bigger than you. Credit to Zolmeister ",
      name: "Pond",
      category_ranking: 40,
      logo_img_url:
        "https://world-id-assets.com/app_e4ae218211628d94881118d63ab9b79a/c89d4beb-947e-4858-a81f-611d01555e97.jpg",
    },
    {
      app_id: "app_2fda3b794bfded1921685908b7c458b3",
      world_app_description: "game, clicker",
      overview:
        "This game is tough! Keep your finger on your phone for as long as you can, or you lose. Winners become legends and get on TV! ",
      name: "HoldIt",
      category_ranking: 41,
      logo_img_url:
        "https://world-id-assets.com/app_2fda3b794bfded1921685908b7c458b3/e10bfe15-2fc1-4bdd-82a9-bcb7d6d06145.png",
    },
    {
      app_id: "app_63656d01dd4c59738b6bfe41ed62297a",
      world_app_description: "Crash the Tower, Claim the Treasure",
      overview:
        "Join thousands of players building a shared tower of blocks that grows increasingly unstable with every move. Purchase a block to drop and maneuver your piece onto the wobbling structure.\n\nWatch as the tower sways and trembles with each successful placement. Every block that lands makes the tower taller, shakier\u2014and the pot bigger. Miss completely? Your block is gone, but the tower stands. But be the player whose block finally brings it all crashing down? You'll walk away with the pot.\n\nThe timing is crucial and the tension is real. With each new addition, the tower becomes more vulnerable. Your perfect placement might be the one that triggers the collapse\u2014and your big payday.\n\nHow steady is your hand? How lucky do you feel? One perfect drop could change everything.\n\nDownload now and turn destruction into profit. ",
      name: "Tower Tumble",
      category_ranking: 42,
      logo_img_url:
        "https://world-id-assets.com/app_63656d01dd4c59738b6bfe41ed62297a/40a58e79-8024-442d-be84-ff128e71dbd1.jpg",
    },
    {
      app_id: "app_1f04a460db1017598662e9b2b7f3fdb3",
      world_app_description: "Flap & Dodge - Earn $FARCADE",
      overview:
        "Earn $FARCADE every day from flapping and dodging pipes in Flapcaster ",
      name: "Flapcaster",
      category_ranking: 43,
      logo_img_url:
        "https://world-id-assets.com/app_1f04a460db1017598662e9b2b7f3fdb3/4669791e-58b7-4b2d-b94a-512f5e3c5a0a.png",
    },
    {
      app_id: "app_57ec63d0c443fc364adb84c46791d044",
      world_app_description: "Playful Purrs, Crypto Rewards.",
      overview:
        "Lightweight cure cloud cat mini program, pet cats to earn cat coins, play puzzle mini-games to win rewards. Unlock new poses to tease cats, double the happiness of daily tasks + social sharing, and easily decompress! ",
      name: "Pet a Cat",
      category_ranking: 44,
      logo_img_url:
        "https://world-id-assets.com/app_57ec63d0c443fc364adb84c46791d044/5fa3e4a7-a201-446b-b7d6-4aa49bfa1938.png",
    },
    {
      app_id: "app_821e7b27df790d0da742c58230d93499",
      world_app_description: "Solve mini-games & win NFTs/Prizes!",
      overview:
        " - Get an exclusive head start before the official 4/20 launch!\n - GUARANTEED exclusive NFT for finishing within 72 hours of launch!\n - +++65 bonus high-value prizes out of 97 left to claim\n - Early Access: Jump into the mini-games immediately\n - Play for Free, and Win Exclusive NFTs / Prizes!\n\nAliya's Awakening: DOGE 2042 is a free dystopian cyberpunk adventure game. Join Aliya and her loyal AI-augmented Shiba Inu puppy, Lua, as they unravel a gritty cyberpunk conspiracy reaching the highest levels of power.\n\nDon't miss out\u2014These exclusive rewards are ONLY available to Worldcoin users through this companion app. Thousands of $ of prizes available to early players. ",
      name: "Aliyas Awakening Mini-Games",
      category_ranking: 45,
      logo_img_url:
        "https://world-id-assets.com/app_821e7b27df790d0da742c58230d93499/fec49346-ab8c-4d8b-aea5-d5e731decb47.png",
    },
    {
      app_id: "app_d4061dd1d8499b88aa7bc8450a385616",
      world_app_description: "Goblinator Terminator",
      overview:
        "Goblin Grinder is a PAY TO WIN game about grinding goblins. Give us your World Coins to pay for revives and lootbox keys.\n\nt.me/GoblinGrinder ",
      name: "Goblin Grinder",
      category_ranking: 46,
      logo_img_url:
        "https://world-id-assets.com/app_d4061dd1d8499b88aa7bc8450a385616/5567d70c-2e15-4ae2-869c-35ca4b1e8d94.png",
    },
    {
      app_id: "app_40d918b450af5b616b7754dbbb2aa977",
      world_app_description: "Play 2048 - Earn $FARCADE",
      overview: "Earn $FARCADE every day from playing 2048. ",
      name: "2048",
      category_ranking: 47,
      logo_img_url:
        "https://world-id-assets.com/app_40d918b450af5b616b7754dbbb2aa977/89440be2-d829-4ec7-accc-8000efeaed05.png",
    },
    {
      app_id: "app_a9047e2a5fce4885ee2b75ba0851fb76",
      world_app_description: "Match 3 - Earn $FARCADE",
      overview: "Earn $FARCADE every day from playing a match 3 style game. ",
      name: "Geo Crush",
      category_ranking: 48,
      logo_img_url:
        "https://world-id-assets.com/app_a9047e2a5fce4885ee2b75ba0851fb76/4ab55a1d-8204-4078-a4cc-cd2a74864e9c.png",
    },
    {
      app_id: "app_298c1409a6012fb1cbe7e0eb492e3195",
      world_app_description: "strategy collection game with PvP",
      overview:
        "First ever free-to-play strategy collection game with PvP battles where all content is Player-owned! ",
      name: "DrakeWars",
      category_ranking: 49,
      logo_img_url:
        "https://world-id-assets.com/app_298c1409a6012fb1cbe7e0eb492e3195/eee8dc59-c1a0-4afd-b1bd-cd00f410d957.jpg",
    },
    {
      app_id: "app_617ef2f6fb474c662b85e2deccf8373c",
      world_app_description: "Knock the door, sell your stuff",
      overview:
        "As a salesman, you knock on the door as hard as you can. Once it opens, you sell your stuff. ",
      name: "DoorTap",
      category_ranking: 50,
      logo_img_url:
        "https://world-id-assets.com/app_617ef2f6fb474c662b85e2deccf8373c/048f8029-6cc8-4e27-ae2d-3c5f333bc286.png",
    },
    {
      app_id: "app_dc8d8b4179aa40499b20efcbd3f5d222",
      world_app_description: "OFFER YOUR BLOOD CHOOSE WISELY",
      overview:
        "Embrace the Darkness: Harvest forbidden power by offering blood sacrifices, but tread carefully\u2014each drop spilled weakens your humanity.\n\nClaim Your Legacy: Unlock vampiric abilities and eternal life, but beware the cost. Will you dominate the night\u2026 or lose yourself to it?\n\nSurvive the Dungeon: Battle monstrous foes, solve twisted riddles, and decide: Extract your riches and flee, or press deeper for greater rewards\u2014and deadlier traps. ",
      name: "Vampire Dungeon",
      category_ranking: 51,
      logo_img_url:
        "https://world-id-assets.com/app_dc8d8b4179aa40499b20efcbd3f5d222/836027be-8af9-43b8-9161-b01bf5b1f03d.png",
    },
    {
      app_id: "app_47624de031399a239e694648c10ba923",
      world_app_description: "Simply enjoy your game",
      overview:
        "Simple games, simple pleasures.\nThere is nothing more enjoyable than a stress-free game.\nYou just click and play and forget all about it. ",
      name: "BitSkyGame",
      category_ranking: 52,
      logo_img_url:
        "https://world-id-assets.com/app_47624de031399a239e694648c10ba923/04a5621c-67bd-4513-ab4c-f1ff442f5e40.png",
    },
    {
      app_id: "app_f0bf3ef645d80f53bcc45097daaae598",
      world_app_description: "Activate Warp Drives, Counteract th",
      overview:
        "Space Adventure Pinball is a WLD Arcade Game.\nA Game that doesn\u2019t need words to be introduced.\nTHE Arcade game par excellence!\nSurvive as long as possible and score as many points as you can! ",
      name: "Pinball Space Adventure",
      category_ranking: 53,
      logo_img_url:
        "https://world-id-assets.com/app_f0bf3ef645d80f53bcc45097daaae598/c3ef4821-3530-4062-bda8-3cdf8688b4a0.png",
    },
    {
      app_id: "app_d6742a7175d03bf862475ae1344d97b5",
      world_app_description: "Shoot. Loot. Repeat.",
      overview:
        "Shoot & Loot is a highly fun and easy - to - play game. The goal is to hit all the targets before your ammo runs out. There are a great number of stages waiting for you, including some boss levels to challenge. ",
      name: "Shoot & Loot",
      category_ranking: 54,
      logo_img_url:
        "https://world-id-assets.com/app_d6742a7175d03bf862475ae1344d97b5/8ddb5b35-d9b4-4fdd-b743-fd57f3af145a.png",
    },
    {
      app_id: "app_976ccdfba5aa4d5b3b31d628d74ea936",
      world_app_description: "Collect & Trade Cards",
      overview:
        "Collect and trade with anime-inspired cards in Anime World TCG.\nBuild your dream deck, trade with real humans, and level up your cards! Climb the ranks in this action-packed mobile card game! ",
      name: "Anime World TCG",
      category_ranking: 55,
      logo_img_url:
        "https://world-id-assets.com/app_976ccdfba5aa4d5b3b31d628d74ea936/61dace69-a0ce-43ef-864a-255a8bd8e470.jpg",
    },
  ],
  Earn: [
    {
      app_id: "app_fa653bbd0e1a4c36aff0e2812876759e",
      world_app_description: "Win up to 2500 WLD daily",
      overview:
        "Cash Earn is the ultimate earnings product where you can win up to 2500 WLD every day. Prizes are distributed every 24 hours. Deposit WLD to enter \u2013 withdraw in full at anytime. Your WLD is never at risk. The more you deposit, the higher your chance to win big.  ",
      name: "Cash Earn",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_fa653bbd0e1a4c36aff0e2812876759e/df172419-9686-4cf5-8364-615a7eca5585.png",
    },
    {
      app_id: "app_795c0423db679ccd64020b91558e0abe",
      world_app_description: "Engage, Verify, Earn.",
      overview:
        "Earn money by discovering the best new products and brands on earth. ",
      name: "EarnOS",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_795c0423db679ccd64020b91558e0abe/f098e391-5b8f-4575-8a29-13b71753c0f3.png",
    },
    {
      app_id: "app_b0d01dd8f2bdfbff06c9e123de487eb8",
      world_app_description: "Earn high rewards on your WLD token",
      overview:
        "Earn high investment returns on your $WLD tokens (25% - 200% variable yearly return). Investment returns are made possible by other traders who pay fees to you, the liquidity provider, when they trade $WLD token. \n\nHow does the yearly return work? \nthe yearly return comes from Uniswap trading fees earned by providing liquidity, either across the full price range (passive) or within a narrow range (concentrated, making capital more efficient). In high-volume markets, frequent trades generate significant fees, allowing APY to exceed 100%. \n\nFor example, if $100 of liquidity was used to facilitate $100 in daily trading volume with a 1% trading fee, then the daily return would be $1 (1%) in rewards. \n\nThe yearly return rate will fluctuate frequently with market conditions. \nThe ratio of USD coin and WLD you hold will also vary based on the current price of WLD (impermanent loss). \n\n\n\n\n\n\n\n ",
      name: "Earn $WLD ",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_b0d01dd8f2bdfbff06c9e123de487eb8/4d0dd3b4-dfde-49ad-8cc9-184d86bf0d60.jpg",
    },
    {
      app_id: "app_04be5c0d2752633311de641688a4c72b",
      world_app_description: "Earn WLD by exercising at home",
      overview:
        "Now you can earn real money while working out! Just do basic exercises like push-ups and squats right at home \u2013 you don't need any gear.\n\nJoin fitness challenges (for example, a challenge to do 50 push-ups every day for 21 days) by staking some Worldcoin (WLD). Then, just do your workout each day in front of your phone camera, while our AI counts your reps for you.\n\nHere's where the extra motivation comes from: if you fail the challenge, you lose your staked WLD, but if you finish the challenge, you get your WLD back plus maybe some extra from other people who dropped out. Plus, for every rep you complete, you get one proof-of-workout token (FIT) too. Who knows what might be in store for this token in the future...  ",
      name: "Squadletics",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_04be5c0d2752633311de641688a4c72b/6fb54b6e-db5e-4f97-8828-5c297b312144.png",
    },
    {
      app_id: "app_85f4c411dc00aadabc96cce7b3a77219",
      world_app_description: "The More You Save, The More You Win",
      overview:
        "PoolTogether is simple - deposit your WLD savings, win prizes, and withdraw any time. PoolTogether is a no-fees, no-loss prize savings app. We're giving away over 100,000+ WLD - so deposit now, because you can't win if you're not in the Pool!\n ",
      name: "PoolTogether",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_85f4c411dc00aadabc96cce7b3a77219/0ec00bd8-b095-44ab-b499-8a90ed7ef66c.png",
    },
    {
      app_id: "app_8542d6a1d600169d41a98c23c655606d",
      world_app_description: "Play. Earn. Train AI.",
      overview:
        "Join thousands of players contributing to AI through fun and rewarding tasks right from your phone! ",
      name: "Sapien",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_8542d6a1d600169d41a98c23c655606d/3e295908-4791-49e8-b4b3-c5dd28fe5748.jpg",
    },
    {
      app_id: "app_5b5a1a91da5ddb972a86d1f740ad895c",
      world_app_description: "Get Paid. Earn Rewards.",
      overview:
        "Join the Craftt Pass rewards and benefits program for your paycheck.\n\nCraftt Pass enables workers to get paid in stablecoins and fiat currencies and earn rewards. Earn points and unlock perks for insurance, travel, everyday savings, and more.\n\nDon't miss out on the upcoming Craftt Rewards Party in Spring/Summer 2025. Get in on the pre-sale now.\n\nOfficial X: @crafttpass\nOfficial TG: @CrafttXYZ ",
      name: "Craftt Pass",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_5b5a1a91da5ddb972a86d1f740ad895c/92ce7ad0-bb24-4ae8-9486-270b7af377aa.png",
    },
    {
      app_id: "app_0ffb335831bc585f54dec2755d917d6a",
      world_app_description: "JUZ: Learn, Earn, and Level Up!",
      overview:
        "JUZ: Unlock the power of knowledge and earn while you learn! Join the global community. Engage in exciting trivia battles, and challenge users from around the world. Joint the world where learning meets competition, and exclusive rewards await those who master new skills and outsmart others. Battle, learn, and earn \u2014 it's time to level up! ",
      name: "JUZ - Earn. Learn",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_0ffb335831bc585f54dec2755d917d6a/f1e7ecac-bedc-4780-81e3-d9dc04079e05.jpg",
    },
    {
      app_id: "app_4c4610d7d45eb20f9804c9365a5a836b",
      world_app_description: "AI Mini-Tasks. Earn Instantly.",
      overview:
        "Gluers connects people with AI-driven job opportunities, mini-tasks, and earning possibilities\u2014all in one powerful app. Whether you're looking for freelance gigs, AI-automated micro-tasks, or AI project flows, Gluers makes it easy to find and complete work instantly. Verified with World ID, users can securely access real jobs, complete them efficiently, and get rewarded with World Coins.  Join Gluers and start earning today! ",
      name: "Gluers People",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_4c4610d7d45eb20f9804c9365a5a836b/6193b153-ffe5-4b3f-a11a-121b700107ce.png",
    },
  ],
  Business: [
    {
      app_id: "app_e1beb4eee66ec6ec4c6684d81b878ff7",
      world_app_description: "Become an Orb Operator",
      overview:
        "Manage your Orb, track revenue, oversee teams and devices, and verify users seamlessly within the Worldcoin ecosystem. Efficient control for operators in one powerful app! ",
      name: "Orb App",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_e1beb4eee66ec6ec4c6684d81b878ff7/3ce3f6b7-1dbb-41f8-9c9c-a71cec636d9d.png",
    },
    {
      app_id: "app_17e9aee55413522124817be4f6e81e42",
      world_app_description: "Get mobile data instantly",
      overview:
        "Buy eSIMs for your favorite destinations and get connected instantly with just a few clicks ",
      name: "eSIM",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_17e9aee55413522124817be4f6e81e42/fc3ff652-ff45-4ffd-83ef-bad240bd1001.png",
    },
    {
      app_id: "app_d9589ab005e18dcf362d2ea26aef669e",
      world_app_description: "The easiest way to pay with tokens",
      overview:
        "Cash Pay is the easiest way to spend and accept crypto. Create payments via QR codes and start accepting crypto payments for your business today. ",
      name: "Cash Pay",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_d9589ab005e18dcf362d2ea26aef669e/22a304f5-f8c5-4865-8714-68b10d0af4a0.png",
    },
    {
      app_id: "app_e4a7e1fafd7c43097627703ffba2cddb",
      world_app_description: "Accept Worldcoin Payments",
      overview:
        "Easily accept Worldcoin payments with the Terminal. Fast, secure, and designed to streamline transactions, letting your customers pay with Worldcoin seamlessly. ",
      name: "Terminal",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_e4a7e1fafd7c43097627703ffba2cddb/47342382-61eb-4d85-992e-8a91e13f88ea.png",
    },
    {
      app_id: "app_df67f69133971e58d0a0d6c75cf0064b",
      world_app_description: "Secure Jobs with Orb Verified",
      overview:
        "\n\nFreelancer \u662f\u5728 WorldApp \u4e2d\u8fd0\u884c\u7684\u4e00\u6b3e\u8ff7\u4f60\u5e94\u7528\uff0c\u65e8\u5728\u4e3a\u901a\u8fc7 Orb \u9a8c\u8bc1\u7684\u7528\u6237\u521b\u5efa\u4e00\u4e2a\u5b89\u5168\u53ef\u9760\u7684\u5e02\u573a\u3002\n\n\nFreelancer es una miniaplicaci\u00f3n que opera dentro de WorldApp, dise\u00f1ada para crear un mercado seguro y confiable para los usuarios verificados por Orb.\n\n\nFreelancer WorldApp \u092e\u0947\u0902 \u0915\u093e\u092e \u0915\u0930\u0928\u0947 \u0935\u093e\u0932\u093e \u090f\u0915 \u092e\u093f\u0928\u0940-\u0910\u092a \u0939\u0948, \u091c\u093f\u0938\u0947 Orb-\u0938\u0924\u094d\u092f\u093e\u092a\u093f\u0924 \u0909\u092a\u092f\u094b\u0917\u0915\u0930\u094d\u0924\u093e\u0913\u0902 \u0915\u0947 \u0932\u093f\u090f \u090f\u0915 \u0938\u0941\u0930\u0915\u094d\u0937\u093f\u0924 \u0914\u0930 \u0935\u093f\u0936\u094d\u0935\u0938\u0928\u0940\u092f \u092c\u093e\u091c\u093c\u093e\u0930 \u092c\u0928\u093e\u0928\u0947 \u0915\u0947 \u0932\u093f\u090f \u0921\u093f\u091c\u093c\u093e\u0907\u0928 \u0915\u093f\u092f\u093e \u0917\u092f\u093e \u0939\u0948\u0964\n\n\nFreelancer \u0647\u0648 \u062a\u0637\u0628\u064a\u0642 \u0645\u0635\u063a\u0631 \u064a\u0639\u0645\u0644 \u062f\u0627\u062e\u0644 WorldApp\u060c \u0645\u0635\u0645\u0645 \u0644\u0625\u0646\u0634\u0627\u0621 \u0633\u0648\u0642 \u0622\u0645\u0646 \u0648\u0645\u0648\u062b\u0648\u0642 \u0644\u0645\u0633\u062a\u062e\u062f\u0645\u064a Orb \u0627\u0644\u0645\u064f\u062d\u064e\u0642\u0642\u064a\u0646.\n\n\nFreelancer \u00e9 um miniaplicativo que opera dentro do WorldApp, projetado para criar um mercado seguro e confi\u00e1vel para usu\u00e1rios verificados pela Orb.\n\n\nFreelancer \u2014 \u044d\u0442\u043e \u043c\u0438\u043d\u0438-\u043f\u0440\u0438\u043b\u043e\u0436\u0435\u043d\u0438\u0435, \u0440\u0430\u0431\u043e\u0442\u0430\u044e\u0449\u0435\u0435 \u0432\u043d\u0443\u0442\u0440\u0438 WorldApp, \u043f\u0440\u0435\u0434\u043d\u0430\u0437\u043d\u0430\u0447\u0435\u043d\u043d\u043e\u0435 \u0434\u043b\u044f \u0441\u043e\u0437\u0434\u0430\u043d\u0438\u044f \u0431\u0435\u0437\u043e\u043f\u0430\u0441\u043d\u043e\u0439 \u0438 \u043d\u0430\u0434\u0451\u0436\u043d\u043e\u0439 \u0442\u043e\u0440\u0433\u043e\u0432\u043e\u0439 \u043f\u043b\u043e\u0449\u0430\u0434\u043a\u0438 \u0434\u043b\u044f \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0435\u0439, \u043f\u0440\u043e\u0432\u0435\u0440\u0435\u043d\u043d\u044b\u0445 \u0447\u0435\u0440\u0435\u0437 Orb.\n\nFreelancer est une mini-application fonctionnant au sein de WorldApp, con\u00e7ue pour cr\u00e9er un march\u00e9 s\u00fbr et fiable pour les utilisateurs v\u00e9rifi\u00e9s par Orb.\n\n\nFreelancer ist eine Mini-App, die innerhalb von WorldApp betrieben wird und darauf ausgelegt ist, einen sicheren und zuverl\u00e4ssigen Marktplatz f\u00fcr Orb-verifizierte Nutzer zu schaffen.\n\n\nFreelancer\u306fWorldApp\u5185\u3067\u52d5\u4f5c\u3059\u308b\u30df\u30cb\u30a2\u30d7\u30ea\u3067\u3001Orb\u8a8d\u8a3c\u6e08\u307f\u30e6\u30fc\u30b6\u30fc\u5411\u3051\u306b\u5b89\u5168\u3067\u4fe1\u983c\u3067\u304d\u308b\u30de\u30fc\u30b1\u30c3\u30c8\u30d7\u30ec\u30a4\u30b9\u3092\u69cb\u7bc9\u3059\u308b\u3088\u3046\u8a2d\u8a08\u3055\u308c\u3066\u3044\u307e\u3059\u3002\n\nFreelancer, WorldApp i\u00e7inde \u00e7al\u0131\u015fan bir mini uygulamad\u0131r ve Orb taraf\u0131ndan do\u011frulanm\u0131\u015f kullan\u0131c\u0131lar i\u00e7in g\u00fcvenli ve g\u00fcvenilir bir pazar yeri olu\u015fturmak \u00fczere tasarlanm\u0131\u015ft\u0131r.\n\n\n\n\n\n\n\n\n\n ",
      name: "Freelancer",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_df67f69133971e58d0a0d6c75cf0064b/2bc62608-afc4-4ada-9f15-0b1b2d30579c.png",
    },
    {
      app_id: "app_d7389cebfa78ca21072403a20135ae4d",
      world_app_description: "Engage, Participate & Earn ",
      overview:
        "Earn tokens by participating in surveys on our platform. Our smart contracts ensure transparent and fair reward distribution, guaranteeing you fair compensation for your contributions. Trust us to provide a rewarding survey experience.\n\nOnce a survey closes, participants on the SurveyBull platform are notified of the available rewards. Claiming these rewards is straightforward - simply interact with this mini app to get started.\n\nInvite others to join the SurveyBull community and reap the rewards through our referral program. ",
      name: "SurveyBull",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_d7389cebfa78ca21072403a20135ae4d/06109f35-a27a-4e64-9af8-f840b8ac801f.png",
    },
    {
      app_id: "app_2643ee14e4743e05ae17aac5534c7e7c",
      world_app_description: "Find your dream job\u2014Easy.",
      overview:
        "A network for humans\u2014where people can find job and gig opportunities to collaborate. Verified users can create private, Orb-verified profiles to access their publications. ",
      name: "Jobs For Humans",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_2643ee14e4743e05ae17aac5534c7e7c/9a05de1e-cd90-4de0-94c1-bdf121358580.png",
    },
    {
      app_id: "app_a11be267c4baa35bca6d18b3cdd6a23c",
      world_app_description: "Find businesses accepting World.",
      overview:
        "Interactive map showcasing businesses around the globe that accept the World cryptocurrency as a form of payment. With an intuitive interface, users can easily find nearby shops, restaurants, and service providers where they can pay using World. At the same time, businesses can effortlessly add their locations and key details, ensuring that customers always have access to up-to-date information. By using World Companies Finder, you save time, gain convenience, and support the growth of the global cryptocurrency community. ",
      name: "World Companies Finder",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_a11be267c4baa35bca6d18b3cdd6a23c/0e10d2bd-8fa5-4232-8db0-9936033400ce.png",
    },
    {
      app_id: "app_d495f6bf966aaa33ce54ebcc6d9a0162",
      world_app_description: "One loyalty app to rule them all!",
      overview:
        "Discounts and coupons with World ID at local World partners! Try products, services, and a range of other benefits.\n\nOur loyalty app is available for all small and medium-sized businesses for free, allowing them to compete with global brands.\n\nWant to join the app? Contact us at: admin@loyall.com\n ",
      name: "LoyAll",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_d495f6bf966aaa33ce54ebcc6d9a0162/3ba9b3b7-06fe-41ad-8d73-edccadcfacf1.png",
    },
  ],
  Productivity: [
    {
      app_id: "app_86794ef02e4fdd6579a937e4a0d858fb",
      world_app_description: "Voting for verified humans",
      overview:
        "Create and share polls, receiving votes from verified, unique humans via World ID for secure, authentic results. Perfect for feedback, decisions, or fun! ",
      name: "Polls",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_86794ef02e4fdd6579a937e4a0d858fb/79dc7a6c-dcb6-45f5-a1cc-e52e59489038.png",
    },
    {
      app_id: "app_e9ff38ec52182a86a2101509db66c179",
      world_app_description: "Create Human-only Telegram Group",
      overview:
        "Invite @world_guard_bot into your Telegram group and it will guard your group with World ID verification! ",
      name: "WorldGuard",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_e9ff38ec52182a86a2101509db66c179/0ec44ee3-93de-4620-b188-e5246e47f66e.png",
    },
    {
      app_id: "app_0779a2d836a4a014279ab7434f98bf7b",
      world_app_description: "Verify humanness offline",
      overview:
        "Create actions that users can verify to prove their humanness and claim physical rewards. Customize verification levels and control how many times actions can be verified. ",
      name: "Human Actions",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_0779a2d836a4a014279ab7434f98bf7b/c03cd1b6-fd2b-429d-a308-9c41b29a5efc.png",
    },
    {
      app_id: "app_9fe9b198f2959d1bb745f81e972e23e6",
      world_app_description: "Earn for being human",
      overview:
        "Connect with global talent instantly through our secure platform that breaks down geographical barriers. Our intuitive solution lets you collaborate seamlessly while building passive income streams. ",
      name: "Work Anywhere",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_9fe9b198f2959d1bb745f81e972e23e6/d07c069c-e210-40fc-b7d2-e93a1ee54543.png",
    },
    {
      app_id: "app_8ef6a5c8af9dce473c8a5b6b3808308f",
      world_app_description: "Your Weather",
      overview:
        "Checking your local weather has never been easier \u2014 Skye brings you everything you need in one clean, easy-to-read screen. ",
      name: "Skye",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_8ef6a5c8af9dce473c8a5b6b3808308f/1c4a57db-4ea0-48c5-b2f9-ff8223d2a699.png",
    },
    {
      app_id: "app_6f33254bc5e69dd5cf9999317a5721f7",
      world_app_description: "Discover your true self.",
      overview:
        "MindVault is designed to help users explore their personalities, preferences, ideologies, among others through engaging assessments. By making self-discovery fun and interactive. ",
      name: "MindVault",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_6f33254bc5e69dd5cf9999317a5721f7/e5efb42c-2d62-4b0f-9f09-56fefe6b7c2b.jpg",
    },
  ],
  AI: [
    {
      app_id: "app_1db2f2a20792e9b422d4825a1b379247",
      world_app_description: "Create images with artist AI",
      overview:
        "TITLES is a free image generator powered by verified AI models from notable artists. Imagine anything using AI, generate for free, and easily share your favorites with friends. ",
      name: "TITLES",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_1db2f2a20792e9b422d4825a1b379247/04d0a06d-2aca-4250-913a-cf66c0f6c7ce.png",
    },
    {
      app_id: "app_5dee2f19cd6eef599eb6ab275a0a7523",
      world_app_description: "Your personal AI",
      overview:
        "Start using Sage, your personal AI companion today. \n\nYou can ask Sage anything, it's powered by the latest AI models.  Sage knows about World, WLD and other miniapps.\n\nDiscover Sage is the #1 AI app on World! Sage is your ultimate AI companion, blending cutting-edge models with a human touch to supercharge your daily life. Whether you're scheduling meetings, seeking instant insights, or exploring new ideas, Sage delivers with unmatched speed and smarts.\n\n\n\nVerified humans unlock the full Sage experience with 20 free messages daily, while unverified users enjoy 10 messages to dive into the action. \n\nWant Sage to know you better? Connect your calendar data to personalize your experience, syncing seamlessly with your schedule for tailored recommendations and reminders that keep you ahead of the curve.\n\nSage doesn't store your data.  ",
      name: "Sage",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_5dee2f19cd6eef599eb6ab275a0a7523/64a89e8e-fc5e-40b0-8cc6-2f50bdc293f1.png",
    },
    {
      app_id: "app_dc38d1977f23660f332458e0c1ca7b58",
      world_app_description: "Earn cash and tokens with AI gigs",
      overview:
        "Launch a side\u2011gig in 60 sec.\n\nChat to AI workers that hunt side-gigs, spot crypto airdrops, draft winning pitches\u2014while you stack Supertokens with every message.\n\n\u2022 Find paid work fast \u2013 tutoring, freelancing, your own business\n\u2022 Earn Supertokens \u2013 convert SUPER to WLD or trade to unlock pro AIs\n\u2022 Auto\u2011post & email \u2013 X, Insta, LinkedIn, Facebook, Gmail\n\u2022 Claim airdrops \u2013 AI finds you new great token World apps\n\nMeet your Superheroes:\n\u2022 Supa \u2013 new airdrops & token claims\n\u2022 Emma \u2013 cold\u2011email\n\u2022 Charlie \u2013 social\u2011media growth\n\u2022 Logan \u2013 instant logos & graphics\n\nNo AI skills needed\u2014just chat & earn. ",
      name: "Superhero: AI Side-Hustle",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_dc38d1977f23660f332458e0c1ca7b58/41faf311-ff54-4946-944c-f1f40c3f57ce.png",
    },
    {
      app_id: "app_ec7ce79e582a7aff416f5b3298cd7a56",
      world_app_description: "Create fun AI Videos",
      overview:
        "Create AI Videos just like you've seen trending on social media. Kiss your crush, make people hug (hug yourself as a child, make Jesus hug you, hug your beloved grandparent who have passed away...) and many other AI video trends coming soon. ",
      name: "VideoGenio: AI Videos",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_ec7ce79e582a7aff416f5b3298cd7a56/dc8439da-bbcf-4a88-9c2a-3375dbe8f8de.png",
    },
    {
      app_id: "app_045f6427f289590148ac2f1c1fa36777",
      world_app_description: "Ask anything",
      overview:
        "Meet Alo-your multi-persona AI buddy! Choose among all the different personalities!\nAsk anything, and watch the conversation come to life ",
      name: "Alo AI",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_045f6427f289590148ac2f1c1fa36777/2598afa4-8515-42d1-92de-f73eb2069ac1.jpg",
    },
    {
      app_id: "app_82fc7befa2f1c7689ec1a9ed441f6226",
      world_app_description: "Card readings, games and NFTs",
      overview:
        "Hello, human. Do you have the courage to probe your past, present, and future? \n\n\u2022\tClaim free TAROT tokens every week\n\u2022\tReceive personalised tarot card readings with unique AI personas\n\u2022\tCollect NFT blessings by meeting secret criteria\n\u2022\tFast predictions on specific questions via quick draws\n\u2022\tJoin the growing community via leaderboards and social profiles\n\u2022\tThis is only the beginning... there is so much more to come\n\nDownload now for a captivating blend of ancient wisdom and cutting-edge technology. ",
      name: "Tarot AI",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_82fc7befa2f1c7689ec1a9ed441f6226/d61e4e5c-3b90-439b-babd-81cf97c408a1.jpg",
    },
    {
      app_id: "app_0411bde5412fe557a6551a15a15f38b2",
      world_app_description: "Capture+remember your conversations",
      overview:
        "Muse is a capture tool and personal AI companion app. Muse captures conversations and builds a personal knowledge base of everything it has heard.\n\nMuse is built to plug into your own sovereign data store ",
      name: "Muse AI Memory Bank",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_0411bde5412fe557a6551a15a15f38b2/1d62d84f-0b4c-4ab8-84c3-602ae41bdbe2.png",
    },
    {
      app_id: "app_c214662d306d0c45abbaac53956075e3",
      world_app_description: "One AI app to rule them all!",
      overview:
        "Access the best AI models within World App. Unleash your imagination with the Image Generators or follow your curiosity with the LLM Chats, including the newest ChatGPT-4.1, Gemini 2.5 Pro and Claude 3.7 Sonnet. Pay only tiny fees on demand. Put your WLD to good use! ",
      name: "The Ring AI",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_c214662d306d0c45abbaac53956075e3/f25e8f25-cb33-49da-bab6-eda62d953a0d.png",
    },
    {
      app_id: "app_404318eb5e6e97937a40b17c890f1ea3",
      world_app_description: "Create music with AI",
      overview:
        "Experience the future of music creation with Melorize, where artificial intelligence meets creative expression. Simply describe the song you envision, and our AI technology will compose an original piece complete with lyrics, melody, and artwork. Want a country song about summer adventures? A rap about overcoming challenges? Or perhaps an indie rock track about city life? Melorize can create it all. Each song is uniquely generated based on your input, ensuring that no two creations are exactly alike. The platform is designed to be intuitive and user-friendly, making music creation accessible to everyone, regardless of their musical background. With Melorize, your musical ideas are just a prompt away from becoming reality. ",
      name: "Melorize: AI Music",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_404318eb5e6e97937a40b17c890f1ea3/0e1dd429-4508-4f3d-a714-7aa7c4775e88.png",
    },
    {
      app_id: "app_7501e98d4a93d6a49aee9336d71788de",
      world_app_description: "Create Fun & Magic AI Photo Art",
      overview:
        "Transform your everyday photos into extraordinary works of art with Picturra AI. Using cutting-edge artificial intelligence, our app offers a magical collection of tools to reimagine your photos in countless creative ways.\n\nCreate stunning anime portraits, fairytale characters, and professional headshots with just one tap. Try our unique features like seeing how you'll look when older, generating AI baby predictions, or bringing your photos to life with animation.\n\nKey Features:\n\u2022 Transform photos into various artistic styles\n\u2022 Create professional AI headshots\n\u2022 Generate fun character variations\n\u2022 Remove and edit photo backgrounds\n\u2022 Enhance and restore old photos\n\u2022 Instant magical photo transformations\n\nWhether you're looking to create professional profile pictures, fun social media content, or preserve precious memories, Picturra AI provides an intuitive and delightful experience for all your creative photo needs. ",
      name: "Picturra AI: Transform Your Photos",
      category_ranking: 10,
      logo_img_url:
        "https://world-id-assets.com/app_7501e98d4a93d6a49aee9336d71788de/72ae1db6-b8fd-4e12-8565-90570ecedae2.png",
    },
    {
      app_id: "app_c223728f049eaaf844bf4107ae81b808",
      world_app_description: "Create personalized cards with AI",
      overview:
        'Create personalized cards with Cardify in seconds! Choose from stunning AI-generated templates, customize text fields with your personal touch, and share beautiful digital cards with friends and family. Perfect for birthdays, holidays, congratulations, or just because. Make every occasion special with Cardify." ',
      name: "Cardify: Personalized AI Cards",
      category_ranking: 11,
      logo_img_url:
        "https://world-id-assets.com/app_c223728f049eaaf844bf4107ae81b808/d5b7012a-952a-49ac-b27d-dd63695cb8fb.png",
    },
    {
      app_id: "app_f61d803c22350091058a3ee2e9e09fa8",
      world_app_description: "The Future of AI at Your Fingertips",
      overview:
        "InfinityAI is a decentralized AI-powered mini-app that gives users seamless access to multiple AI models, including GPT-4o, Gemini, DeepSeek, and Claude Sonnet. Integrated with Worldcoin (WLD) for authentication and payments, users can securely interact with AI while spending credits. Whether you need content generation, coding assistance, or general knowledge, InfinityAI provides limitless possibilities in a user-friendly experience. New users receive free trial credits, making AI more accessible than ever. ",
      name: "InfinityAI",
      category_ranking: 12,
      logo_img_url:
        "https://world-id-assets.com/app_f61d803c22350091058a3ee2e9e09fa8/8db23b62-d2e6-4656-bd77-14f1fb41ce46.jpg",
    },
  ],
  Social: [
    {
      app_id: "app_0844e90773d1ec26c4d47e111879f4c4",
      world_app_description: "Collect clips & win USDC",
      overview: "Win USDC by collecting the best clips before they get famous ",
      name: "Aqua",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_0844e90773d1ec26c4d47e111879f4c4/6289408b-29dd-4d82-960d-0cf2c9b1f35d.png",
    },
    {
      app_id: "app_a1a7fb139d05d20c50af7ba30b453f91",
      world_app_description: "The proof you\u2019re human \u2014 on social ",
      overview:
        "UMAN \u2013 Officially Human.\n\nConnect socials. Verify humanity. Share proof.\n\nNo bots, no captchas\u2014just you. ",
      name: "Uman",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_a1a7fb139d05d20c50af7ba30b453f91/cee795f4-90fc-4442-ba7c-48723c3f8072.jpg",
    },
    {
      app_id: "app_2378a08c66b2d599eca345e0cf605a3c",
      world_app_description: "Claim Pixels, Unite Worlds",
      overview:
        "Pixel World unites players worldwide to co-create evolving pixel art on a shared canvas. Join real-time collaboration, blend cultural designs, and build a dynamic community masterpiece\u2014one pixel at a time. ",
      name: "Pixel World",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_2378a08c66b2d599eca345e0cf605a3c/a5d3fd82-cb35-4166-86ed-65b50212f9b4.png",
    },
    {
      app_id: "app_d11f6453b6675157bd01d889a0728ed0",
      world_app_description: "One D-ID to Own it All",
      overview:
        "ForU AI is changing how people take control of their digital identity. We\u2019re building the largest AI D-ID platform, allowing users to manage and monetize their data. By unifying fragmented data into personalized avatars, ForU AI leads the way in a decentralized Data Economy. \n\nSign up now to create a World ID for your ForU Account and take control of your data! ",
      name: "ForU AI",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_d11f6453b6675157bd01d889a0728ed0/5c5d70cc-a27c-4190-81da-926afbbf8d6c.png",
    },
    {
      app_id: "app_743a529bcdabe926b060fc1c26d38fcb",
      world_app_description: "World Streaming",
      overview:
        "Vivo is a live broadcast, heard by everyone at the same time. Right now, it\u2019s music, tomorrow, it could be something else. Tap & Tune in with other real people.  ",
      name: "Vivo",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_743a529bcdabe926b060fc1c26d38fcb/beb39c8c-fa87-4986-a90b-be212caae876.png",
    },
    {
      app_id: "app_9872915a7a53ef9915cceb646a5cf06d",
      world_app_description: "Real People. Real Connections.",
      overview:
        "Meritt: The Human-Only Social Network\n\nWelcome to the Last Truly Human Space on the Internet.\n\nIn a world drowning in AI-generated content, deep fakes, and bot interactions, Meritt stands as a beacon of authenticity. We're not just another social platform \u2013 we're a sanctuary of genuine human connection.\n\nOur Promise: 100% Verified Humans Only\n\nNo Bots\nNo AI Profiles\nNo Fake Accounts\n\nEvery single user on Meritt has been verified through World's groundbreaking iris-scanning technology. When you interact on Meritt, you're connecting with a real person \u2013 guaranteed.\n\nHow It Works\nVerification: Complete World's iris scan\nDaily Interaction Points: Receive a daily allocation of points\nMeaningful Engagement: Spend points on likes, comments, and shares\n\nWhy Meritt?\n\nAuthentic Connections: Talk to real humans, not algorithms\nQuality over Quantity: Limited daily points make every interaction meaningful\nSafe Community: Verified human ecosystem eliminates digital noise\n\nReclaim the internet. Connect with real people. ",
      name: "Meritt",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_9872915a7a53ef9915cceb646a5cf06d/529830a1-7515-48cd-a5bc-1e374a23eba5.png",
    },
    {
      app_id: "app_14667489aa3b47eff9937c45aafa3988",
      world_app_description: "Surveys for verified humans",
      overview:
        "Answer Survey only if you are a verified human. Results are shown only when your answer is submitted. ",
      name: "AskHumans",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_14667489aa3b47eff9937c45aafa3988/eed5130e-eb94-471b-91e1-d6188377ec19.png",
    },
    {
      app_id: "app_d1d53da49a19e867e8d2a280ad7d2e5f",
      world_app_description: "The smart Ring that earns.",
      overview:
        "The first AI-empowered wellness ring that rewards your wellness journey.\n\nSpecial rewards are ready for the WORLDCOIN community\u2014claim yours now! ",
      name: "CUDIS Ring",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_d1d53da49a19e867e8d2a280ad7d2e5f/4df9a57d-2d93-468a-8f77-ee6c852cd63e.png",
    },
    {
      app_id: "app_84d0d5db191bfd3a0690eef8a0823a4e",
      world_app_description: "Read about World",
      overview:
        "The thinking, ideas, and technology behind a more human economic system. ",
      name: "World Blog",
      category_ranking: 9,
      logo_img_url:
        "https://world-id-assets.com/app_84d0d5db191bfd3a0690eef8a0823a4e/58cc8238-e49b-4dc0-9e4a-5da417d92e0a.png",
    },
    {
      app_id: "app_fd8485420e315e6c1b745e168b9044b5",
      world_app_description: "Human Town Square - Meme - Bot free",
      overview:
        "CapSha is your gateway to an internet social meme game experiment, where only real humans thrive. \n\nNo bots, no fake accounts - just an authentic onchain social game designed to build a strong and genuine community. \n\nJoin the human only internet town square, start collecting CapSha and let's create together the future of decentralized social platform.  ",
      name: "CapSha",
      category_ranking: 10,
      logo_img_url:
        "https://world-id-assets.com/app_fd8485420e315e6c1b745e168b9044b5/0da0063f-5024-456f-8e70-d0c5278ef690.jpg",
    },
    {
      app_id: "app_a01bf4b7b47315f64e6d29de9cf6f2f3",
      world_app_description: "Connected, even when disconnected",
      overview:
        "Your identity, information, and privacy should not be connected to your connection. Say hello to portable user profiles that work without internet or an active phone connection.\n\nOfflineID is the foundational block of Offline Protocol, a pioneering censorship resistant ecosystem of payments, messaging, and information sharing apps that use resilient alternative networking without internet or telecom signals. ",
      name: "OfflineID",
      category_ranking: 11,
      logo_img_url:
        "https://world-id-assets.com/app_a01bf4b7b47315f64e6d29de9cf6f2f3/4408d228-c5a4-4ff2-9677-8df609df5d4c.png",
    },
    {
      app_id: "invites",
      world_app_description: "Invite friends and get rewards",
      overview: "Invite your friends and get rewards ",
      name: "Invites",
      category_ranking: 12,
      logo_img_url:
        "https://world-id-assets.com/app_432af83feb4051e72fd7ee682f365c39/0c8b6545-5cdf-4ab4-b670-9114affccde9.png",
    },
  ],
  Other: [
    {
      app_id: "app_e40049a2b0b344c63754a954b84308df",
      world_app_description: "Drops and Collections",
      overview: "Explore merch collections and drops for humans only ",
      name: "World Shop",
      category_ranking: 1,
      logo_img_url:
        "https://world-id-assets.com/app_e40049a2b0b344c63754a954b84308df/e10c79a3-9e73-4bf9-a868-f6da9b7cb8ff.png",
    },
    {
      app_id: "app_77638c005fe267b8995d307dbb3c9bbe",
      world_app_description: "Pick a card or trust the marble",
      overview:
        "Pick a card to unveil your aura\u2019s secrets or let the magic marble reveal what fate has in store. Just bring some Stardust\u2014because even destiny doesn\u2019t work for free. ",
      name: "Fortune Teller",
      category_ranking: 2,
      logo_img_url:
        "https://world-id-assets.com/app_77638c005fe267b8995d307dbb3c9bbe/528f24fb-858c-4c9a-8d57-d70d371d4a1b.png",
    },
    {
      app_id: "app_bc7cf1ec2ae3388f416263138ab5b1c3",
      world_app_description: "Attest to receive tokens",
      overview:
        "L2 Faucet allows users to attest their devices to receive testnet tokens directly without bridging, social verification or tasklists. A product by Automata Network.  ",
      name: "L2 Faucet",
      category_ranking: 3,
      logo_img_url:
        "https://world-id-assets.com/app_bc7cf1ec2ae3388f416263138ab5b1c3/f9658ba9-3103-48d1-b8f9-97b1f7ecc181.png",
    },
    {
      app_id: "app_d29cf8cfeea14e69f286af1803e296d2",
      world_app_description: "Purchase hotels with WLD and USDC.e",
      overview:
        "Search, compare, and book hotels instantly. Pay with WLD and USDC.e. ",
      name: "Hotels Cryptorefills",
      category_ranking: 4,
      logo_img_url:
        "https://world-id-assets.com/app_d29cf8cfeea14e69f286af1803e296d2/0f317db3-ee76-4ef7-bceb-61c89dbb5f4c.jpg",
    },
    {
      app_id: "app_17add0ea360017d9ed307f8913dd4a0e",
      world_app_description: "Buy flights with WLD and USDC.e",
      overview:
        "Search for flights from hundreds of different airlines. Pay with WLD and USDC.e. ",
      name: "Flights Cryptorefills",
      category_ranking: 5,
      logo_img_url:
        "https://world-id-assets.com/app_17add0ea360017d9ed307f8913dd4a0e/a239c7ea-b175-4ef1-81ec-a664b7119649.jpg",
    },
    {
      app_id: "app_718b1068295a80c4f095dd69798d161e",
      world_app_description: "Buy gift cards and mobile top ups",
      overview:
        "Cash out your WLD and USDC.e for real goods and services as gift cards, mobile top up and eSIMs. ",
      name: "Cryptorefills",
      category_ranking: 6,
      logo_img_url:
        "https://world-id-assets.com/app_718b1068295a80c4f095dd69798d161e/8d097d08-6e1a-43d3-ae8d-bc9949d7bba2.jpg",
    },
    {
      app_id: "app_e3d8e924de647edcf0cca6afd4273235",
      world_app_description: "PagoLinea Servicios",
      overview:
        "Con WorldCoin, pagar tus servicios en Argentina es r\u00e1pido y sencillo. Desde la plataforma, puedes abonar en un solo clic tus facturas de Claro y Personal, recargar tu l\u00ednea m\u00f3vil o incluso cargar tu tarjeta del Subte de Buenos Aires. La interfaz intuitiva y la seguridad de WorldCoin garantizan transacciones r\u00e1pidas y confiables, eliminando la necesidad de largos tr\u00e1mites o m\u00faltiples plataformas. Gracias a WorldCoin, mantenerte al d\u00eda con tus pagos cotidianos es m\u00e1s f\u00e1cil que nunca, ahorr\u00e1ndote tiempo y esfuerzo.\n\n\n\n\n ",
      name: "PagoLinea Servicios",
      category_ranking: 7,
      logo_img_url:
        "https://world-id-assets.com/app_e3d8e924de647edcf0cca6afd4273235/29f437bf-dca7-4637-9db9-05e72dc7ae36.jpg",
    },
    {
      app_id: "app_e8288209fbe1fc4a1b80619e925a79bd",
      world_app_description: "Interactive courses about World",
      overview:
        "Learn about World through interactive courses. Get to know how the tools in World Network can help you including World ID, World App, Worldcoin (WLD) and the Orb. ",
      name: "Learn",
      category_ranking: 8,
      logo_img_url:
        "https://world-id-assets.com/app_e8288209fbe1fc4a1b80619e925a79bd/2d565374-e944-49ac-b283-8193eb908ed8.png",
    },
  ],
};

export const appIdToApp = (appId: string) => {
  return Object.values(miniapps)
    .flat()
    .find((app) => app.app_id === appId);
};
