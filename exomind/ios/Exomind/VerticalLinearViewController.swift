import UIKit
import SnapKit

class VerticalLinearViewController: UIViewController {
    var scrollView: UIScrollView!
    var scrollViewContainer: UIView!

    var linearViews = [UIView]()
    var lastBound: Bool = false

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createScrollView()
    }

    func createScrollView() {
        self.scrollView = UIScrollView()
        self.view.addSubview(self.scrollView)
        self.scrollView.snp.makeConstraints { (make) in
            make.center.equalTo(self.view.snp.center)
            make.size.equalTo(self.view.snp.size)
        }

        self.scrollViewContainer = UIView()
        self.scrollView.addSubview(self.scrollViewContainer)
        self.scrollViewContainer.snp.makeConstraints { (make) in
            make.width.equalTo(self.scrollView.snp.width)
            make.top.equalTo(self.scrollView.snp.top)
            make.bottom.equalTo(self.scrollView.snp.bottom)
        }
    }

    func addLinearView(_ view: UIView, minHeight: CGFloat? = nil) {
        self.scrollViewContainer.addSubview(view)
        view.snp.makeConstraints { (make) in
            if let previous = self.linearViews.last {
                make.top.equalTo(previous.snp.bottom)
            } else {
                make.top.equalTo(self.scrollViewContainer.snp.top)
            }
            make.left.equalTo(self.scrollViewContainer.snp.left)
            make.width.equalTo(self.scrollViewContainer.snp.width)

            if let minHeight = minHeight {
                make.height.greaterThanOrEqualTo(minHeight)
            }
        }
        self.linearViews.append(view)
    }

    override func viewWillAppear(_ animated: Bool) {
        if (!self.lastBound) {
            if let last = self.linearViews.last {
                last.snp.makeConstraints { (make) in
                    make.bottom.equalTo(self.scrollViewContainer.snp.bottom)
                }
            }
            self.lastBound = true
        }
    }

    deinit {
        print("VerticalLinearViewController > Deinit")
    }
}
