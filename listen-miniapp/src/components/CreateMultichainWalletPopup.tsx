import { AnimatePresence, motion, PanInfo, useAnimation } from "framer-motion";
import { useEffect } from "react";
import { useWalletCreate } from "../hooks/useWalletCreate";
import { GradientOutlineButtonMoreRounded } from "./GradientOutlineButtonMoreRounded";
import { OutlineButton } from "./OutlineButton";
import { Rectangle } from "./Rectangle";

export const CreateMultichainWalletPopup = ({
  isVisible,
  onClose,
}: {
  isVisible: boolean;
  onClose: () => void;
}) => {
  const controls = useAnimation();
  const { handleCreate, getButtonText, isCreating, solanaReady } =
    useWalletCreate();

  useEffect(() => {
    if (isVisible) {
      controls.start({ y: 0, opacity: 1 });
    } else {
      controls.start({ y: "100%", opacity: 0 });
    }
  }, [isVisible, controls]);

  const handleDragEnd = (
    _: MouseEvent | TouchEvent | PointerEvent,
    info: PanInfo
  ) => {
    const threshold = 100;
    if (info.offset.y > threshold) {
      onClose();
    } else {
      controls.start({ y: 0 });
    }
  };

  const handleCreateAndClose = async () => {
    await handleCreate();
    onClose();
  };

  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="fixed inset-0 z-50 flex items-end backdrop-blur-sm bg-black/50"
          onClick={(e) => {
            if (e.target === e.currentTarget) onClose();
          }}
        >
          <motion.div
            drag="y"
            dragConstraints={{ top: 0 }}
            dragElastic={0.2}
            onDragEnd={handleDragEnd}
            initial={{ y: "100%" }}
            animate={controls}
            exit={{ y: "100%" }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
            className="relative w-full max-w-md mx-auto bg-[#151518] border border-[#2D2D2D] rounded-t-[24px] shadow-xl pb-9"
          >
            <div className="flex justify-center pt-2">
              <Rectangle />
            </div>
            <div className="p-6">
              <div className="flex flex-col items-center justify-center gap-6 mb-5">
                <div className="font-space-grotesk text-2xl leading-8 tracking-[-0.03em] text-center align-middle font-normal text-white">
                  Create multi-chain account to trade more coins?
                </div>
              </div>
              <div className="flex flex-row justify-center gap-4">
                <GradientOutlineButtonMoreRounded
                  text={getButtonText()}
                  onClick={handleCreateAndClose}
                  disabled={isCreating || !solanaReady}
                />
                <OutlineButton
                  text="Later"
                  onClick={onClose}
                  disabled={isCreating}
                />
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
