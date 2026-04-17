melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e2 e4 f4 g2 a2 g4 c4 c4 b4 c2 r2 \break

  % Line 2
  c2 c4 c4 b4 a2 g2 fis4 g2 r2 \break

  % Line 3
  g2 a4 a4 b2 g2 c4 b4 a4 a4 g2 r2 \break

  % Line 4
  e2 g4 a4 c4 b2 a2 gis4 a2 r2 \break

  % Line 5
  e2 f4 f4 e2 d2 g4 g4 a2 b2 c2 r2 \break

  % Line 6
  c2 b4 a4 d2 c2 b4 b4 a1 \bar "|."
}
