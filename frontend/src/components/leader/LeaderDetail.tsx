import { useLeaderStore } from '../../store/leaderStore';
import LeaderBasicInfo from './LeaderBasicInfo';
import LeaderTimelineChart from './LeaderTimelineChart';
import LeaderComparison from './LeaderComparison';
import type { LeaderBoardItem } from '../../types/leader';

interface LeaderDetailProps {
  stock: LeaderBoardItem | null;
}

function LeaderDetail({ stock }: LeaderDetailProps) {
  const { comparedStocks } = useLeaderStore();

  return (
    <div>
      <LeaderBasicInfo stock={stock} />
      <LeaderTimelineChart stock={stock} />
      {comparedStocks.length > 0 && <LeaderComparison />}
    </div>
  );
}

export default LeaderDetail;
