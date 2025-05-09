import { WorldMiniApp } from "../prompts/miniapps";

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full min-h-[100px] bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px]">
      {children}
    </div>
  );
};

const AppLogo = ({ src, alt }: { src: string; alt: string }) => {
  return (
    <img
      src={src}
      alt={alt}
      className="w-[56px] h-[56px] border-[1px] border-[#404040] rounded-full"
    />
  );
};

const RedirectArrow = () => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    className="text-[#868686] hover:text-white transition-colors"
  >
    <path
      d="M7 17L17 7M17 7H8M17 7V16"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

export function WorldAppRedirect({ app }: { app: WorldMiniApp }) {
  return (
    <Container>
      <div className="flex flex-row py-4 px-4 items-center w-full justify-between">
        <div className="flex flex-row items-center">
          <AppLogo src={app.logo_img_url} alt={app.name} />
          <div className="flex flex-col p-2">
            <div className="flex flex-row items-center space-x-2">
              <div className="font-space-grotesk font-normal text-2xl leading-8 tracking-[-0.03em] text-center align-middle">
                {app.name}
              </div>
            </div>
            <div className="font-dm-sans font-light text-[14px] leading-[16px] tracking-[0%] align-middle text-[#868686]">
              {app.world_app_description}
            </div>
          </div>
        </div>
        <a
          href={`worldapp://mini-app?app_id=${app.app_id}`}
          className="p-2 hover:bg-[#1e1e21] rounded-full transition-colors"
          title="Open in World App"
        >
          <RedirectArrow />
        </a>
      </div>
    </Container>
  );
}
