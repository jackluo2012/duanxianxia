import { useEffect, RefObject } from 'react';

interface AudioPlayerProps {
  audioRef: RefObject<HTMLAudioElement | null>;
  onEnded?: () => void;
  onError?: () => void;
}

function AudioPlayer({ audioRef, onEnded, onError }: AudioPlayerProps) {
  useEffect(() => {
    // 创建音频元素
    const audio = new Audio();

    // 设置事件监听
    const handleEnded = () => {
      onEnded?.();
    };

    const handleError = () => {
      onError?.();
    };

    audio.addEventListener('ended', handleEnded);
    audio.addEventListener('error', handleError);

    // 将音频元素保存到ref
    (audioRef as any).current = audio;

    // 清理函数
    return () => {
      audio.pause();
      audio.src = '';
      audio.removeEventListener('ended', handleEnded);
      audio.removeEventListener('error', handleError);
    };
  }, [onEnded, onError]);

  return null; // 这是一个无UI的组件，只负责管理音频播放
}

export default AudioPlayer;
