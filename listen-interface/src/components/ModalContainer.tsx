import { useModal } from "../contexts/ModalContext";
import { BuySellModal } from "./BuySellModal";

export function ModalContainer() {
  const { buySellModalState, closeBuySellModal } = useModal();

  return buySellModalState.isOpen && buySellModalState.asset ? (
    <BuySellModal
      isOpen={buySellModalState.isOpen}
      onClose={closeBuySellModal}
      action={buySellModalState.action}
      asset={buySellModalState.asset}
    />
  ) : null;
}
