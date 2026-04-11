melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c4 e4 g4 c,4 f4 a4 a4( g4) e8( f8 g8 c,8 f4) e8( f8) e4( d4) c2 \break

  % Line 2
  f4 g4 a4 g4 f4 e4 e4( d4) e8( f8 g8 c,8 f4) e8( f8) e4( d4) c2 \break

  % Line 3
  b'4 c4 d4 g,4 c4 d4 e2 b8( c8 d8 g,8 c8) b8( c8) b4( a4) g2 \break

  % Line 4
  g8( a8) b8( g8) c4 e,4 f4 a4 a4( g4) c8( b8 c8 g8 a8 b8) c8( d8) c4( b4) c2 \bar "|."
}
