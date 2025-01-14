import { Background } from "./components/Background";
import { Chat } from "./components/Chat";
import { Header } from "./Header";

function App() {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 py-20">
        <Chat />
      </div>
    </div>
  );
}

export default App;
