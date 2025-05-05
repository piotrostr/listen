import { Token } from "../lib/types";

const PercentageChange = ({ pct_change }: { pct_change: number }) => {
  const isPositive = pct_change >= 0;
  const formattedChange = `${isPositive ? "+" : ""}${pct_change.toFixed(2)}%`;

  return (
    <div
      className={`
        flex justify-center items-center p-2 rounded-full w-14 h-7 font-dm-sans
        ${isPositive ? "bg-pump-green-bg" : "bg-pump-red-bg"}
      `}
    >
      <span
        className={`
          text-xs font-normal leading-3
          ${isPositive ? "text-pump-green" : "text-pump-red"}
        `}
      >
        {formattedChange}
      </span>
    </div>
  );
};

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full h-[196px] bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px]">
      {children}
    </div>
  );
};

const TokenImage = ({ src, alt }: { src: string; alt: string }) => {
  return (
    <img
      src={src}
      alt={alt}
      className="w-[56px] h-[56px] border-[1px] border-[#404040] rounded-full"
    />
  );
};

const ChartLine = ({
  ema_price_ticks,
  pct_change,
}: {
  ema_price_ticks: { price: number }[];
  pct_change: number;
}) => {
  // Skip if no data
  if (!ema_price_ticks?.length) return null;

  const isPositive = pct_change >= 0;
  const lineColor = isPositive ? "#8DFC63" : "#8A5EFB";
  const gradientStartColor = isPositive ? "#8DFC63" : "#8057FB";
  const gradientEndColor = isPositive ? "#8DFC63" : "#F72777";

  // Get min and max for scaling
  const prices = ema_price_ticks.map((tick) => tick.price);
  const minPrice = Math.min(...prices);
  const maxPrice = Math.max(...prices);
  const priceRange = maxPrice - minPrice;

  // Create points for the path
  const points = ema_price_ticks.map((tick, i) => {
    const x = (i / (ema_price_ticks.length - 1)) * 358;
    const y = 105 - ((tick.price - minPrice) / priceRange) * 98;
    return `${x},${y}`;
  });

  const linePath = `M${points.join(" L")}`;
  const fillPath = `${linePath} L358,105 L0,105 Z`;

  return (
    <svg
      width="100%"
      height="105"
      viewBox="0 0 358 105"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className="w-full"
    >
      <defs>
        <linearGradient
          id="chartGradient"
          x1="179"
          y1="5"
          x2="179"
          y2="105"
          gradientUnits="userSpaceOnUse"
        >
          <stop stopColor={gradientStartColor} />
          <stop offset="1" stopColor={gradientEndColor} stopOpacity="0" />
        </linearGradient>
      </defs>

      <path d={fillPath} fill="url(#chartGradient)" fillOpacity="0.16" />

      <path
        d={linePath}
        stroke={lineColor}
        strokeWidth="4"
        strokeLinecap="round"
        fill="none"
      />
    </svg>
  );
};

export function TokenDisplay({}: { token: Token }) {
  const { metadata, price_info } = mockData;
  const name =
    metadata?.mpl?.name.length > 15 ? metadata.mpl.symbol : metadata.mpl.name;
  return (
    <Container>
      <div className="flex flex-row p-4 items-center">
        <TokenImage
          src={metadata?.mpl?.ipfs_metadata?.image}
          alt={metadata?.mpl?.name}
        />
        <div className="flex flex-col p-2">
          <div className="flex flex-row items-center space-x-2">
            <div className="font-space-grotesk font-normal text-2xl leading-8 tracking-[-0.03em] text-center align-middle">
              {metadata?.mpl?.name}
            </div>
            {price_info?.pct_change && (
              <PercentageChange pct_change={price_info?.pct_change} />
            )}
          </div>
          <div className="font-dm-sans font-light text-[14px] leading-[16px] tracking-[0%] align-middle text-[#868686]">
            {price_info?.latest_price.toFixed(8)}
          </div>
        </div>
      </div>
      {price_info?.ema_price_ticks && (
        <ChartLine
          ema_price_ticks={price_info.ema_price_ticks}
          pct_change={price_info.pct_change || 0}
        />
      )}
    </Container>
  );
}

const mockData: Token = {
  metadata: {
    mint: "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump",
    mpl: {
      name: "listen-rs",
      symbol: "listen",
      uri: "https://ipfs.io/ipfs/QmaujLDEbH9i8jHehteAZbxQKd9cV7Khf6EirJE5Fr7W4d",
      ipfs_metadata: {
        createdOn: "https://pump.fun",
        description: "blazingly fast actions for AI agents in Rust",
        image:
          "https://ipfs.io/ipfs/QmQB8PKqR8jfJUux8nwsreoQjW9Ja5S8xyGtcPR5P4tuf4",
        name: "listen-rs",
        symbol: "listen",
        showName: true,
        twitter: "https://x.com/piotreksol",
        website: "https://www.listen-rs.com/",
      },
    },
    spl: {
      mint_authority: null,
      supply: 999992147351515,
      decimals: 6,
      is_initialized: true,
      freeze_authority: null,
    },
  },
  price_info: {
    latest_price: 0.0038610194365547666,
    ema_price_ticks: [
      { price: 0.004068820016596712 },
      { price: 0.00406957206855634 },
      { price: 0.004083545202200752 },
      { price: 0.0040964646046644515 },
      { price: 0.0041127566086990325 },
      { price: 0.004119043169076037 },
      { price: 0.004121911951393864 },
      { price: 0.00411857735974051 },
      { price: 0.004112264112656314 },
      { price: 0.004104601578564721 },
      { price: 0.004102790390001883 },
      { price: 0.004096564058344763 },
      { price: 0.004095558668878416 },
      { price: 0.004097805709200751 },
      { price: 0.004103499222801629 },
      { price: 0.0041081181593505085 },
      { price: 0.004112731995756838 },
      { price: 0.004113220529026542 },
      { price: 0.0041003651135448375 },
      { price: 0.004095180578363352 },
      { price: 0.004090715280566758 },
      { price: 0.004076808289870512 },
      { price: 0.004063777100880428 },
      { price: 0.004050816356582853 },
      { price: 0.004040111644879561 },
      { price: 0.0040335959798486086 },
      { price: 0.004021088370468145 },
      { price: 0.004013960527456574 },
      { price: 0.004008053527692293 },
      { price: 0.003999250933474188 },
      { price: 0.0039865266429183295 },
      { price: 0.003978852227831333 },
      { price: 0.003963875247353552 },
      { price: 0.003954453490694861 },
      { price: 0.003948487757454424 },
      { price: 0.003940231448077574 },
      { price: 0.003933003403772091 },
      { price: 0.0039234433111209575 },
      { price: 0.0039051398930292675 },
      { price: 0.003903070929548642 },
      { price: 0.0039034302731150907 },
      { price: 0.0039046255181411464 },
      { price: 0.003906030367925903 },
      { price: 0.0039056050469074115 },
      { price: 0.0039124900181136115 },
      { price: 0.0039055498162751125 },
      { price: 0.003896091552840313 },
      { price: 0.003890177801690392 },
      { price: 0.0038847870784323125 },
      { price: 0.003880225864628639 },
      { price: 0.0038757489277275467 },
      { price: 0.003872484643205626 },
      { price: 0.003862182604212458 },
      { price: 0.003863657237669172 },
      { price: 0.003865235068183905 },
      { price: 0.0038600403775593643 },
      { price: 0.003861248750180407 },
      { price: 0.0038854591402540375 },
      { price: 0.003902580008514816 },
      { price: 0.003922706395937016 },
      { price: 0.003937530777971492 },
      { price: 0.003937795425205585 },
      { price: 0.003933838955334021 },
      { price: 0.003921614271748951 },
      { price: 0.003909583994635194 },
      { price: 0.003901831846428126 },
      { price: 0.0038908712360129225 },
      { price: 0.003882524925808987 },
      { price: 0.003872479291446505 },
      { price: 0.003867070901576971 },
      { price: 0.0038366998322439984 },
      { price: 0.0038065630818155476 },
      { price: 0.003777034626686589 },
      { price: 0.0037530663821028223 },
      { price: 0.003736383435431032 },
      { price: 0.0037237555241369205 },
      { price: 0.0037145732220279855 },
      { price: 0.0037144409760135645 },
      { price: 0.003712931329487123 },
      { price: 0.003710778659463858 },
      { price: 0.0037092843394045468 },
      { price: 0.003708423592513909 },
      { price: 0.003708042981970007 },
      { price: 0.0036929072081701457 },
      { price: 0.0036765385651758996 },
      { price: 0.0036661388916833116 },
      { price: 0.003659953987395298 },
      { price: 0.003655568721200614 },
      { price: 0.00365493111759685 },
      { price: 0.003651686545753328 },
      { price: 0.003653645114937268 },
      { price: 0.003651704246451874 },
      { price: 0.0036456940006161136 },
      { price: 0.00364428919415708 },
      { price: 0.00364337402076752 },
      { price: 0.003642451590363076 },
      { price: 0.003638741993419503 },
      { price: 0.003630932055399836 },
      { price: 0.0036145006137258463 },
      { price: 0.0036056954586932 },
      { price: 0.0035970765847249445 },
      { price: 0.003582473083030369 },
      { price: 0.0035679522202979874 },
      { price: 0.003540889059253317 },
      { price: 0.003527886962717396 },
      { price: 0.003515042241712051 },
      { price: 0.0035513947633463864 },
      { price: 0.00362426972400327 },
      { price: 0.003688103606479397 },
      { price: 0.0037208887037640686 },
      { price: 0.0037328024493417902 },
      { price: 0.0037396552769311496 },
      { price: 0.00375231564577266 },
      { price: 0.003762648976000683 },
      { price: 0.0037665173636130312 },
      { price: 0.0037675809548016895 },
      { price: 0.0037710609758138383 },
      { price: 0.003770876372890441 },
      { price: 0.003768610946728048 },
      { price: 0.003763328594641384 },
      { price: 0.0037502837080798453 },
      { price: 0.0037388513697075017 },
      { price: 0.0037256247538959325 },
      { price: 0.0037126911462349987 },
      { price: 0.0036906053950848993 },
      { price: 0.0036681887548049748 },
      { price: 0.003656940727155669 },
      { price: 0.0036426997981092294 },
      { price: 0.003627441797495009 },
      { price: 0.0036169482367216454 },
      { price: 0.003609412611278232 },
      { price: 0.003604837823413487 },
      { price: 0.00359764635069306 },
      { price: 0.0035913710432296077 },
      { price: 0.0035901039884432846 },
      { price: 0.0035851938849970305 },
      { price: 0.0035784552467099497 },
      { price: 0.0035713675152353667 },
      { price: 0.0035626300622992953 },
      { price: 0.0035577582352775846 },
      { price: 0.003548976036823505 },
      { price: 0.0035420170290108455 },
      { price: 0.0035348714944019263 },
      { price: 0.0035314640522549306 },
      { price: 0.003531402442566683 },
      { price: 0.0035365905631058437 },
      { price: 0.0035380501134291225 },
      { price: 0.0035335782789662523 },
      { price: 0.0035248181382329675 },
      { price: 0.003513138520585145 },
      { price: 0.0035080627812831247 },
      { price: 0.003509177965893795 },
      { price: 0.00351979951276423 },
      { price: 0.003533424630502957 },
      { price: 0.00354245942014449 },
      { price: 0.0035498881346530554 },
      { price: 0.0035685590775413616 },
      { price: 0.003582686267170003 },
      { price: 0.0035919739321678577 },
      { price: 0.0036142864484212337 },
      { price: 0.003640426213352719 },
      { price: 0.003658447530565459 },
      { price: 0.00367874936615077 },
      { price: 0.00369244875873262 },
      { price: 0.003733231501700564 },
      { price: 0.003766532449985191 },
      { price: 0.003790009246398466 },
      { price: 0.0038031150445142093 },
      { price: 0.0038117647520012356 },
      { price: 0.00381989285397343 },
      { price: 0.003825745583427364 },
      { price: 0.0038378278722708175 },
      { price: 0.003870057107135085 },
      { price: 0.00396615085096111 },
      { price: 0.004039676900697965 },
      { price: 0.004092454078697893 },
      { price: 0.004137342526544589 },
      { price: 0.004170551098439099 },
      { price: 0.004195197269524099 },
      { price: 0.004220541318192362 },
      { price: 0.0042382063789516995 },
      { price: 0.004251238591203142 },
      { price: 0.00425562384585348 },
      { price: 0.004239005223173725 },
      { price: 0.004225533699757222 },
      { price: 0.004213097772289051 },
      { price: 0.004205048143665685 },
      { price: 0.0041994904020722455 },
      { price: 0.004176456667089543 },
      { price: 0.004159463594615836 },
      { price: 0.004130901265113688 },
      { price: 0.00411246442337429 },
      { price: 0.004077908335805154 },
      { price: 0.0039998523904896804 },
      { price: 0.003955458151684836 },
      { price: 0.003934268544496574 },
      { price: 0.003920003395278663 },
      { price: 0.0038934985434633156 },
      { price: 0.0038758933987757043 },
      { price: 0.0038610194365547666 },
    ],
    price_ticks_timeframe: "15m",
    total_volume: 279591.0494546523,
    pct_change: -6.752164237187403,
    period: "last 54.2 hours",
  },
};
