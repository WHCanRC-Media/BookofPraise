melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a4 c4 b2 a2 g4 f4 e2 d2 r2 \break

  % Line 2
  f2 f4 e4 f2 g2 a2 r2 \break

  % Line 3
  a2 c4 c4 b2 d2 a2 r2 \break

  % Line 4
  d,2( a'2) b2 c4 b4 a4 a4 g2 f2 r2 \break

  % Line 5
  a2 a4 a4 d2 c2 b4 a2 gis4 a2 r2 \break

  % Line 6
  f2 g4 f4 e2 d1 \bar "|."
}
