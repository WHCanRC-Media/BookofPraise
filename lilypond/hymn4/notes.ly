melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 c4 b4 a2 g2 e4 f4 g4 g4 f2 e2 \break

  % Line 2
  g2 a4 b4 c2 a2 b4 c4 d4 g,4 a2 g2 \break

  % Line 3
  e2 d4 c4 g'2 g2 a4 c4 b4 a4 a4( gis4) a2 \break

  % Line 4
  g2 f4 e4 d2 c2 \bar "|."
}
