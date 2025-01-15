export const Background = () => {
  return (
    <div className="fixed inset-0 -z-10 h-screen w-screen bg-black">
      <div className="absolute inset-0 bg-black opacity-80" />
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-purple-700/30 rounded-full mix-blend-screen filter blur-3xl animate-blob" />
        <div className="absolute top-1/3 right-1/4 w-96 h-96 bg-blue-700/30 rounded-full mix-blend-screen filter blur-3xl animate-blob animation-delay-2000" />
        <div className="absolute bottom-1/4 left-1/3 w-96 h-96 bg-indigo-700/30 rounded-full mix-blend-screen filter blur-3xl animate-blob animation-delay-4000" />
        <div className="absolute bottom-1/3 right-1/3 w-96 h-96 bg-pink-700/30 rounded-full mix-blend-screen filter blur-3xl animate-blob animation-delay-6000" />
      </div>
    </div>
  );
};
