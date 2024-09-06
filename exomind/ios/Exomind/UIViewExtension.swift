import UIKit
import SnapKit

extension UIView {

    // http://stackoverflow.com/questions/10167266/how-to-set-cornerradius-for-only-top-left-and-top-right-corner-of-a-uiview
    func roundCorners(_ corners: UIRectCorner, radius: CGFloat) {
        let path = UIBezierPath(roundedRect: self.bounds, byRoundingCorners: corners, cornerRadii: CGSize(width: radius, height: radius))
        let mask = CAShapeLayer()
        mask.path = path.cgPath
        self.layer.mask = mask
    }

    // http://stackoverflow.com/questions/24370061/assign-xib-to-the-uiview-in-swift
    class func loadFromNibNamed(_ nibNamed: String, bundle: Bundle? = nil) -> UIView? {
        UINib(
                nibName: nibNamed,
                bundle: bundle
        ).instantiate(withOwner: nil, options: nil)[0] as? UIView
    }

    func addBorderBottomView() {
        let border = UIView()
        border.backgroundColor = UIColor.black.withAlphaComponent(0.1)
        self.addSubview(border)
        border.snp.makeConstraints { (make) in
            make.left.equalTo(self.snp.left).offset(10)
            make.right.equalTo(self.snp.right)
            make.height.equalTo(1)
            make.bottom.equalTo(self.snp.bottom)
        }
    }
}
