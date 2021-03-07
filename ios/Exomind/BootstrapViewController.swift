import UIKit
import WebKit
import Exocore

class BootstrapViewController: UIViewController {
    @IBOutlet weak var errorLabel: UILabel!
    @IBOutlet weak var pinLabel: UILabel!
    @IBOutlet weak var configText: UITextView!

    var disco: Discovery?
    var onDone: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()

        startDiscovery()
        self.refreshNodeConfig()
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

    @IBAction func onSave(_ sender: Any) {
        do {
            self.disco = nil
            let configJson = self.configText.text ?? ""
            let config = try Exocore_Core_LocalNodeConfig(jsonString: configJson)
            let node = try LocalNode.from(config: config)
            ExocoreUtils.saveNode(node: node)
            startDiscovery()
        } catch {
            self.errorLabel.text = error.localizedDescription
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

    private func refreshNodeConfig() {
        let config = try? ExocoreUtils.node?.config().jsonString()
        self.configText.text = self.jsonPrettyPrint(config ?? "")
    }

    private func jsonPrettyPrint(_ jsonStr: String) -> String {
        if let data = jsonStr.data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: data, options: .mutableContainers),
           let jsonData = try? JSONSerialization.data(withJSONObject: json, options: .prettyPrinted) {
            return String(decoding: jsonData, as: UTF8.self)
        } else {
            return jsonStr
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
