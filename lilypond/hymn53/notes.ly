melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c'4 c4 c4 g8( a8) b4 c8( b8) a4 g4 \break

  % Line 2
  c4 b4 a4 g4 a4 f4( d4) c4 \break

  % Line 3
  c'4 c4 c4 g8( a8) b4 c8 b8 a4 g4 \break

  % Line 4
  c4 b4 a4 g4 a4 f4( d4) c4 \break

  % Line 5
  c4 g'4 a4 g4 fis4 g2 \break

  % Line 6
  c,4 g'4 g4 a4 b4 c2 \break

  % Line 7
  b4 c4 b4 a4 a4 g2 \break

  % Line 8
  a4 a4 g4 a4 f4 e2 \break

  % Line 9
  c'4 b4 a4 g4 a4 f4 d4 c4 \bar "|."
}
