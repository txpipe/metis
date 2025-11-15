import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

// Extender solo una vez
dayjs.extend(relativeTime);

export default dayjs;
