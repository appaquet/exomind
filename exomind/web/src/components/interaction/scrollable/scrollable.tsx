import React from 'react';
import './scrollable.less';
import PropTypes from 'prop-types';


interface IProps {
    children?: React.ReactNode;
    nbItems: number;
    onNeedMore?: () => void;
    loadMoreItems: number,
    initialTopInset?: number,
}

export default class Scrollable extends React.Component<IProps> {
    private lastNbItems = 0;
    private loadingMore = false;
    private scrollableElem: React.RefObject<HTMLDivElement> = React.createRef();

    static propTypes = {
        nbItems: PropTypes.number.isRequired,
        onNeedMore: PropTypes.func,
        loadMoreItems: PropTypes.number,
        initialTopInset: PropTypes.number,
    };

    componentDidMount(): void {
        if (this.scrollableElem) {
            this.scrollableElem.current.scrollTop = this.props.initialTopInset ?? 0;
        }
    }

    render(): React.ReactNode {
        if (this.props.nbItems != this.lastNbItems) {
            this.lastNbItems = this.props.nbItems;
            this.loadingMore = false;
        }

        return (
            <div className="scrollable" ref={this.scrollableElem} onScroll={this.handleCollectionScroll}>
                {this.props.children}
            </div>
        );
    }

    private handleCollectionScroll = () => {
        const ul = this.scrollableElem.current;
        if (ul && !this.loadingMore && this.props.nbItems > 0 && this.props.loadMoreItems) {
            const scrollPosition = ul.scrollTop;
            const scrollHeight = (ul.scrollHeight - ul.clientHeight);
            const itemAvgHeight = scrollHeight / this.props.nbItems;
            const nbItemsToScroll = (scrollHeight - scrollPosition) / itemAvgHeight;

            if (nbItemsToScroll <= this.props.loadMoreItems) {
                if (this.props.onNeedMore) {
                    this.props.onNeedMore();
                    this.loadingMore = true;
                }
            }
        }
    };
}

// Inspired from https://htmldom.dev/check-if-an-element-is-visible-in-a-scrollable-container/
export function isVisibleWithinScrollable(ele: HTMLElement, padding = 50): boolean {
    let scrollable: HTMLElement | null = ele.parentElement;
    for (let i = 0; i < 20; i++) {
        if (!scrollable) {
            return false;
        }

        if (scrollable.className.indexOf('scrollable') >= 0) {
            break;
        }

        scrollable = scrollable.parentElement;
    }

    const eleTop = ele.offsetTop;
    const eleBottom = eleTop + ele.clientHeight;
    const containerTop = scrollable.scrollTop;
    const containerBottom = containerTop + scrollable.clientHeight;

    if (eleBottom + padding > containerBottom) {
        return false;
    } else if (eleTop - padding < containerTop) {
        return false;
    }

    return true;
}