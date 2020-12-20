import UIKit
import WebKit
import Exocore

class BootstrapViewController: UIViewController {
    @IBOutlet weak var errorLabel: UILabel!
    @IBOutlet weak var pinLabel: UILabel!

    var disco: Discovery?
    var onDone: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()

        startDiscovery()
    }

    @IBAction func onReset(_ sender: Any) {
        do {
            self.disco = nil
            let node = try LocalNode.generate()
            ExocoreUtils.saveNode(node: node)
            startDiscovery()
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }

    @IBAction func onClose(_ sender: Any) {
        if ExocoreUtils.nodeHasCell {
            self.onDone?()
        } else {
            self.errorLabel.text = "Node needs to join a cell"
        }
    }

    private func startDiscovery() {
        guard let node = ExocoreUtils.node else {
            self.errorLabel.text = "No node configured, but should had one"
            return
        }

        self.errorLabel.text = nil

        do {
            self.disco = try Discovery()
            self.disco?.joinCell(localNode: node, callback: { [weak self] (stage) in
                guard let this = self else { return }

                DispatchQueue.main.async {
                    switch stage {
                    case .pin(let pin):
                        this.pinLabel.text = formatPin(pin)

                    case .success(let newNode):
                        ExocoreUtils.saveNode(node: newNode)
                        this.tryBoot()

                    case .error(let err):
                        this.errorLabel.text = err.localizedDescription
                    }
                }
            })
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }

    private func tryBoot() {
        do {
            try ExocoreUtils.bootNode()
            self.onDone?()
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }
}


private func formatPin(_ pin: UInt32) -> String {
    let strPin = pin.description

    var ret = ""
    for (i, char) in strPin.enumerated() {
        if i == 3 || i == 6 {
            ret += " "
        }
        ret += String(char)
    }
    return ret
}
