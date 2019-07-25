
import UIKit

class ViewController: UIViewController {
    var context: OpaquePointer?

    override func viewDidLoad() {
        super.viewDidLoad()

        let res = exocore_context_new();
        if res.status == UInt8(ExocoreError_Success.rawValue) {
            self.context = res.context
        }
    }

    @IBAction func buttonClick(_ sender: Any) {
        exocore_send_query(self.context, "hello world")
    }
}
