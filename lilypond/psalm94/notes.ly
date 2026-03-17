melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e2 a4 a4 g2 c2 a4 b4 c2 b2 r2 \break

  % Line 2
  b2 c4 a4 b4 g4 a4 c4 b2 a2 r2 \break

  % Line 3
  b2 c4 g4 a2 g2 f4 f4 e2 r2 \break

  % Line 4
  e2 e4 d4 e4 g4 g4 fis4 g2 r2 \break

  % Line 5
  a2 g4 e4 f4 a4 a4 gis4 a2 r2 \break

  % Line 6
  e2 e4 a4 g2 e2 f4 f4 e1 \bar "|."
}
