import UIKit
import SnapKit
import Exocore

class EntityCreationViewController: ModalGridViewController {
    var parentId: EntityId?
    var callback: ((ActionResult) -> Void)?

    convenience init(parentId: EntityId?, callback: ((ActionResult) -> Void)? = nil) {
        self.init(nibName: nil, bundle: nil)
        self.parentId = parentId
        self.callback = callback
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createView()
    }

    func createView() {
        let actions = Actions.forEntityCreation(self.parentId)
        let items: [GridIconsViewItem] = actions.compactMap { action in
            guard let icon = action.icon else {
                return nil
            }

            let label = action.label.replacingOccurrences(of: "Create ", with: "").capitalized

            return GridIconsViewItem(label: label, icon: icon) { item in
                action.execute { res in
                    self.callback?(res)
                }
                self.close()
            }
        }

        let view = GridIconsView(items: items)
        view.squarePerRow = 3
        view.initView()
        self.view.addSubview(view)
        view.snp.makeConstraints { (make) in
            make.size.equalTo(self.view.snp.size)
            make.center.equalTo(self.view.snp.center)
        }
    }
}
