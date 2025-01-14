import { Background } from "./components/Background";
import { Chat } from "./components/Chat";
import { Header } from "./components/Header";
import { Portfolio } from "./components/Portfolio";

function App() {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 py-20">
        <div className="flex flex-col lg:flex-row gap-4 max-w-7xl mx-auto px-4">
          <div className="flex-1">
            <Chat />
          </div>
          <div className="w-full lg:w-80">
            <Portfolio />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
