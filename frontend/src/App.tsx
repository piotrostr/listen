import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { motion } from "framer-motion";
import { Header } from "./Header";
import { Background } from "./Background";

const letterAnimation = {
  initial: { y: 20, opacity: 0 },
  animate: { y: 0, opacity: 1 },
  transition: { duration: 0.5 }
};

const staggerContainer = {
  animate: {
    transition: {
      staggerChildren: 0.1
    }
  }
};

const FeatureCard = ({ title, features, index }) => {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
      className="p-8 rounded-xl bg-gray-800/50 backdrop-blur hover:bg-gray-700/50 transition-all duration-300 transform hover:-translate-y-2"
    >
      <motion.div 
        whileHover={{ scale: 1.02 }}
        className="mb-6"
      >
        <motion.h3 
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.2 }}
          className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent"
        >
          {title}
        </motion.h3>
        <div className="h-1 w-16 bg-gradient-to-r from-blue-400 to-purple-400 mt-2 rounded-full" />
      </motion.div>
      
      <div className="space-y-4">
        {features.map((feature, idx) => (
          <motion.div
            key={idx}
            initial={{ opacity: 0, x: -10 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 * idx }}
            className="flex items-start space-x-3 group"
          >
            <motion.span 
              className="text-blue-400 text-lg mt-1"
              whileHover={{ scale: 1.2 }}
            >
              •
            </motion.span>
            <p className="text-gray-300 font-medium leading-relaxed group-hover:text-gray-100 transition-colors">
              {feature}
            </p>
          </motion.div>
        ))}
      </div>
    </motion.div>
  );
};

const AnimatedTitle = ({ text, className = "" }) => {
  // Split text into words
  const words = text.split(" ");
  
  return (
    <div className={`flex flex-wrap justify-center gap-x-2 ${className}`}>
      {words.map((word, wordIndex) => (
        <motion.div
          key={wordIndex}
          className="inline-flex"
          whileHover={{ scale: 1.05 }}
          transition={{ type: "spring", stiffness: 400, damping: 10 }}
        >
          {word.split("").map((char, charIndex) => (
            <motion.span
              key={`${wordIndex}-${charIndex}`}
              className="inline-block"
              whileHover={{ 
                y: -5,
                color: "#60A5FA",
                transition: { type: "spring", stiffness: 500 }
              }}
            >
              {char}
            </motion.span>
          ))}
        </motion.div>
      ))}
    </div>
  );
};

const App = () => {
  const features = [
    {
      title: "Token Analysis & Checks",
      features: [
        "Comprehensive holder analysis",
        "Ownership concentration checks",
        "Mint authority verification",
        "Program authority analysis",
        "Metadata validation"
      ]
    },
    {
      title: "Real-time Monitoring",
      features: [
        "Transaction tracking",
        "Configurable worker threads",
        "Adjustable buffer sizes",
        "Prometheus metrics integration",
        "WebSocket subscription handling"
      ]
    },
    {
      title: "Advanced Swapping",
      features: [
        "Multi-platform swap execution",
        "Support for Pump.fun",
        "Jupiter V6 API integration",
        "Direct Raydium interaction",
        "Custom slippage settings"
      ]
    },
    {
      title: "Price Tracking",
      features: [
        "Real-time price monitoring",
        "PubSub subscription system",
        "Webhook integration",
        "Price alert configuration",
        "Historical data tracking"
      ]
    },
    {
      title: "Token Management",
      features: [
        "Custom wallet generation",
        "Batch token account closing",
        "Token sweeping functionality",
        "ATA Sweeps",
        "Balance consolidation"
      ]
    },
    {
      title: "Performance Tools",
      features: [
        "Transaction profiling",
        "RPC benchmarking",
        "Real-time priority fee",
        "Memory usage tracking",
        "Latency monitoring"
      ]
    }
  ];

  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 max-w-6xl mx-auto px-4 py-20">
        {/* Hero Section */}
        <motion.div 
          className="text-center mb-32 mt-10"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <motion.img
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            whileHover={{ 
              rotate: 360,
              scale: 1.1,
              transition: { duration: 0.8, ease: "easeInOut" }
            }}
            transition={{ type: "spring", stiffness: 200, damping: 20 }}
            src="/listen.svg"
            alt="listen"
            className="w-32 h-32 mx-auto mb-12 cursor-pointer"
          />
          
          <motion.div className="mb-6">
            <AnimatedTitle 
              text="listen-rs"
              className="text-6xl md:text-7xl font-bold" 
            />
          </motion.div>

          <motion.p 
            className="text-2xl text-gray-300 mb-8"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.4 }}
          >
            <AnimatedTitle text="blazingly fast actions for AI agents in Rust" />
          </motion.p>
          
          <motion.div 
            className="mt-8 flex flex-row justify-center items-center space-x-6"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.6 }}
          >
            <motion.a 
              href="https://github.com/piotrostr/listen" 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.95 }}
              className="transform transition-transform"
            >
              <img
                src="https://img.shields.io/github/stars/piotrostr/listen?style=social"
                alt="GitHub Stars"
                className="shadow-lg"
              />
            </motion.a>
            <motion.a 
              href="https://github.com/piotrostr/listen/blob/main/LICENSE" 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.95 }}
              className="transform transition-transform"
            >
              <img
                src="https://img.shields.io/github/license/piotrostr/listen"
                alt="License"
                className="shadow-lg"
              />
            </motion.a>
          </motion.div>
          
          <motion.div 
            className="mt-12 flex justify-center"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.8 }}
          >
            <motion.div 
              className="rounded-lg overflow-hidden shadow-2xl"
              whileHover={{ 
                scale: 1.02,
                boxShadow: "0 20px 25px -5px rgba(0, 0, 0, 0.2), 0 10px 10px -5px rgba(0, 0, 0, 0.1)"
              }}
            >
              <SyntaxHighlighter
                language="bash"
                style={{
                  ...vscDarkPlus,
                  'code[class*="language-"]': {
                    color: "#fff",
                    padding: "1.5rem",
                  },
                }}
              >
                {`cargo add --git https://github.com/piotrostr/listen listen`}
              </SyntaxHighlighter>
            </motion.div>
          </motion.div>
        </motion.div>

        {/* Features Grid */}
        <motion.div 
          variants={staggerContainer}
          initial="initial"
          animate="animate"
          className="grid md:grid-cols-2 lg:grid-cols-3 gap-10"
        >
          {features.map((feature, index) => (
            <FeatureCard
              key={index}
              title={feature.title}
              features={feature.features}
              index={index}
            />
          ))}
        </motion.div>
      </div>
    </div>
  );
};

export default App;